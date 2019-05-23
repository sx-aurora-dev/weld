//! Code generation for the merger builder type.

use llvm_sys;

use std::ffi::CString;

use crate::ast::BinOpKind;
use crate::ast::ScalarKind;
use crate::ast::Type::{Scalar, Simd};
use crate::error::*;

use self::llvm_sys::core::*;
use self::llvm_sys::prelude::*;
use self::llvm_sys::LLVMTypeKind;

use crate::codegen::c::llvm_exts::*;
use crate::codegen::c::numeric::gen_binop;
use crate::codegen::c::CodeGenExt;
use crate::codegen::c::LLVM_VECTOR_WIDTH;
use crate::codegen::c::CContextRef;

const SCALAR_INDEX: u32 = 0;
const VECTOR_INDEX: u32 = 1;

/// The merger type.
pub struct Merger {
    pub merger_ty: LLVMTypeRef,
    pub name: String,
    pub elem_ty: LLVMTypeRef,
    pub c_elem_ty: String,
    pub scalar_kind: ScalarKind,
    pub op: BinOpKind,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    ccontext: CContextRef,
    new: Option<LLVMValueRef>,
    merge: Option<LLVMValueRef>,
    vmerge: Option<LLVMValueRef>,
    result: Option<LLVMValueRef>,
}

impl CodeGenExt for Merger {
    fn module(&self) -> LLVMModuleRef {
        self.module
    }

    fn context(&self) -> LLVMContextRef {
        self.context
    }

    fn ccontext(&self) -> CContextRef {
        self.ccontext
    }
}

impl Merger {
    pub unsafe fn define<T: AsRef<str>>(
        name: T,
        op: BinOpKind,
        elem_ty: LLVMTypeRef,
        c_elem_ty: &str,
        scalar_kind: ScalarKind,
        context: LLVMContextRef,
        module: LLVMModuleRef,
        ccontext: CContextRef,
    ) -> Merger {
        let c_name = CString::new(name.as_ref()).unwrap();
        let mut layout = [elem_ty, LLVMVectorType(elem_ty, LLVM_VECTOR_WIDTH)];
        let merger = LLVMStructCreateNamed(context, c_name.as_ptr());
        LLVMStructSetBody(merger, layout.as_mut_ptr(), layout.len() as u32, 0);
        Merger {
            name: c_name.into_string().unwrap(),
            op,
            merger_ty: merger,
            elem_ty,
            c_elem_ty: c_elem_ty.to_string(),
            scalar_kind,
            context,
            module,
            ccontext,
            new: None,
            merge: None,
            vmerge: None,
            result: None,
        }
    }

    pub unsafe fn gen_new(
        &mut self,
        builder: LLVMBuilderRef,
        init: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.new.is_none() {
            let ret_ty = self.merger_ty;
            let c_ret_ty = &self.name.clone();
            let mut arg_tys = [self.elem_ty];
            let c_arg_tys = [self.c_elem_ty.clone()];
            let name = format!("{}.new", self.name);
            let (function, builder, _, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name);

            let identity = self.binop_identity(self.op, self.scalar_kind)?;
            let mut vector_elems = [identity; LLVM_VECTOR_WIDTH as usize];
            let vector_identity =
                LLVMConstVector(vector_elems.as_mut_ptr(), vector_elems.len() as u32);
            let one = LLVMBuildInsertValue(
                builder,
                LLVMGetUndef(self.merger_ty),
                LLVMGetParam(function, 0),
                SCALAR_INDEX,
                c_str!(""),
            );
            let result =
                LLVMBuildInsertValue(builder, one, vector_identity, VECTOR_INDEX, c_str!(""));

            LLVMBuildRet(builder, result);

            self.new = Some(function);
            LLVMDisposeBuilder(builder);
        }

        let mut args = [init];
        Ok(LLVMBuildCall(
            builder,
            self.new.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }

    /// Builds the `Merge` function and returns a reference to the function.
    ///
    /// The merge function is similar for the scalar and vector varianthe `gep_index determines
    /// which one is generated.
    unsafe fn gen_merge_internal(
        &mut self,
        name: String,
        arguments: &mut [LLVMTypeRef],
        c_arguments: &[String],
        gep_index: u32,
    ) -> WeldResult<LLVMValueRef> {
        let ret_ty = LLVMVoidTypeInContext(self.context);
        let c_ret_ty = &self.void_c_type();
        let (function, fn_builder, _, _) = self.define_function(ret_ty, c_ret_ty, arguments, c_arguments, name);

        LLVMExtAddAttrsOnFunction(self.context, function, &[LLVMExtAttribute::AlwaysInline]);

        // Load the vector element, apply the binary operator, and then store it back.
        let elem_pointer =
            LLVMBuildStructGEP(fn_builder, LLVMGetParam(function, 0), gep_index, c_str!(""));
        let elem = LLVMBuildLoad(fn_builder, elem_pointer, c_str!(""));
        let result = gen_binop(
            fn_builder,
            self.op,
            elem,
            LLVMGetParam(function, 1),
            &Simd(self.scalar_kind),
        )?;
        LLVMBuildStore(fn_builder, result, elem_pointer);
        LLVMBuildRetVoid(fn_builder);
        LLVMDisposeBuilder(fn_builder);
        Ok(function)
    }

    pub unsafe fn gen_merge(
        &mut self,
        llvm_builder: LLVMBuilderRef,
        builder: LLVMValueRef,
        value: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        let vectorized = LLVMGetTypeKind(LLVMTypeOf(value)) == LLVMTypeKind::LLVMVectorTypeKind;
        if vectorized {
            if self.vmerge.is_none() {
                let mut arg_tys = [
                    LLVMPointerType(self.merger_ty, 0),
                    LLVMVectorType(self.elem_ty, LLVM_VECTOR_WIDTH as u32),
                ];
                let c_arg_tys = [
                    self.pointer_c_type(&self.name),
                    self.simd_c_type(&self.c_elem_ty, LLVM_VECTOR_WIDTH as u32),
                ];
                let name = format!("{}.vmerge", self.name);
                self.vmerge = Some(self.gen_merge_internal(name, &mut arg_tys, &c_arg_tys, VECTOR_INDEX)?);
            }
            let mut args = [builder, value];
            Ok(LLVMBuildCall(
                llvm_builder,
                self.vmerge.unwrap(),
                args.as_mut_ptr(),
                args.len() as u32,
                c_str!(""),
            ))
        } else {
            if self.merge.is_none() {
                let mut arg_tys = [LLVMPointerType(self.merger_ty, 0), self.elem_ty];
                let c_arg_tys = [self.pointer_c_type(&self.name), self.c_elem_ty.clone()];
                let name = format!("{}.merge", self.name);
                self.merge = Some(self.gen_merge_internal(name, &mut arg_tys, &c_arg_tys, SCALAR_INDEX)?);
            }
            let mut args = [builder, value];
            Ok(LLVMBuildCall(
                llvm_builder,
                self.merge.unwrap(),
                args.as_mut_ptr(),
                args.len() as u32,
                c_str!(""),
            ))
        }
    }

    pub unsafe fn gen_result(
        &mut self,
        llvm_builder: LLVMBuilderRef,
        builder: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.result.is_none() {
            let ret_ty = self.elem_ty;
            let c_ret_ty = &self.c_elem_ty.clone();
            let mut arg_tys = [LLVMPointerType(self.merger_ty, 0)];
            let c_arg_tys = [self.pointer_c_type(&self.name)];
            let name = format!("{}.result", self.name);
            let (function, fn_builder, _, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name);

            // Load the scalar element, apply the binary operator, and then store it back.
            let builder_pointer = LLVMGetParam(function, 0);
            let scalar_pointer =
                LLVMBuildStructGEP(fn_builder, builder_pointer, SCALAR_INDEX, c_str!(""));
            let mut result = LLVMBuildLoad(fn_builder, scalar_pointer, c_str!(""));

            let vector_pointer =
                LLVMBuildStructGEP(fn_builder, builder_pointer, VECTOR_INDEX, c_str!(""));
            let vector = LLVMBuildLoad(fn_builder, vector_pointer, c_str!(""));

            for i in 0..LLVM_VECTOR_WIDTH {
                let vector_element =
                    LLVMBuildExtractElement(fn_builder, vector, self.i32(i as i32), c_str!(""));
                result = gen_binop(
                    fn_builder,
                    self.op,
                    result,
                    vector_element,
                    &Scalar(self.scalar_kind),
                )?;
            }

            LLVMBuildRet(fn_builder, result);

            self.result = Some(function);
            LLVMDisposeBuilder(fn_builder);
        }
        let mut args = [builder];
        Ok(LLVMBuildCall(
            llvm_builder,
            self.result.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }
}
