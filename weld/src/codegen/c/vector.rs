//! Extensions to generate vectors.
//!
//! This module provides a wrapper interface for methods and utilities on vector types. Other
//! modules use it for vector-related functionality or operators over vectors.
//!
//! Many of the methods here are marked as `alwaysinline`, so method calls on vectors usually have
//! no overhead. Because of the fundamental nature of vectors, their layout is always fixed to be a
//! tuple (pointer, size). Other modules may use knowledge of this layout to, e.g., provide vector
//! operators over pointers (the methods here are over loaded structs).

use llvm_sys;

use std::ffi::CString;
use code_builder::CodeBuilder;

use crate::ast::Type;
use crate::error::*;

use super::llvm_exts::LLVMExtAttribute::*;
use super::llvm_exts::*;

use self::llvm_sys::core::*;
use self::llvm_sys::prelude::*;

use super::intrinsic::Intrinsics;
use super::CodeGenExt;
use super::CGenerator;
use super::LLVM_VECTOR_WIDTH;
use super::c_u64_type;

use crate::codegen::c::CContextRef;

/// Index of the pointer into the vector data structure.
pub const POINTER_INDEX: u32 = 0;
/// Index of the size into the vector data structure.
pub const SIZE_INDEX: u32 = 1;

/// Extensions for generating methods on vectors.
///
/// This provides convinience wrappers for calling methods on vectors. The `vector_type` is the
/// vector type (not the element type).
pub trait VectorExt {
    unsafe fn gen_new(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        size: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef>;
    unsafe fn gen_clone(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef>;
    unsafe fn gen_at(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: LLVMValueRef,
        index: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef>;
    unsafe fn c_gen_at(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: &str,
        index: &str,
    ) -> WeldResult<String>;
    unsafe fn gen_vat(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: LLVMValueRef,
        index: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef>;
    unsafe fn gen_size(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef>;
    unsafe fn c_gen_size(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: &str,
    ) -> WeldResult<String>;
    unsafe fn gen_extend(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: LLVMValueRef,
        size: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef>;
}

impl VectorExt for CGenerator {
    unsafe fn gen_new(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        size: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.gen_new(builder, &mut self.intrinsics, run, size)
        } else {
            unreachable!()
        }
    }

    unsafe fn gen_clone(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vector: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.gen_clone(builder, &mut self.intrinsics, vector, run)
        } else {
            unreachable!()
        }
    }

    unsafe fn gen_at(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vector: LLVMValueRef,
        index: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.gen_at(builder, vector, index)
        } else {
            unreachable!()
        }
    }

    unsafe fn c_gen_at(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vector: &str,
        index: &str,
    ) -> WeldResult<String> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.c_gen_at(builder, vector, index)
        } else {
            unreachable!()
        }
    }

    unsafe fn gen_vat(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vector: LLVMValueRef,
        index: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.gen_vat(builder, vector, index)
        } else {
            unreachable!()
        }
    }

    unsafe fn gen_size(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vector: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.gen_size(builder, vector)
        } else {
            unreachable!()
        }
    }

    unsafe fn c_gen_size(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vector: &str,
    ) -> WeldResult<String> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.c_gen_size(builder, vector)
        } else {
            unreachable!()
        }
    }

    unsafe fn gen_extend(
        &mut self,
        builder: LLVMBuilderRef,
        vector_type: &Type,
        vec: LLVMValueRef,
        size: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if let Type::Vector(ref elem_type) = *vector_type {
            let methods = self.vectors.get_mut(elem_type).unwrap();
            methods.gen_extend(builder, &mut self.intrinsics, run, vec, size)
        } else {
            unreachable!()
        }
    }
}

/// A vector type and its associated methods.
pub struct Vector {
    pub vector_ty: LLVMTypeRef,
    pub name: String,
    pub elem_ty: LLVMTypeRef,
    pub c_elem_ty: String,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    ccontext: CContextRef,
    new: Option<LLVMValueRef>,
    c_new: String,
    clone: Option<LLVMValueRef>,
    c_clone: String,
    at: Option<LLVMValueRef>,
    c_at: String,
    vat: Option<LLVMValueRef>,
    c_vat: String,
    size: Option<LLVMValueRef>,
    c_size: String,
    slice: Option<LLVMValueRef>,
    c_slice: String,
    extend: Option<LLVMValueRef>,
    c_extend: String,
}

impl CodeGenExt for Vector {
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

impl Vector {
    /// Define a new vector type with the given element type.
    ///
    /// This function only inserts a definition for the vector, but does not generate any new code.
    pub unsafe fn define<T: AsRef<str>>(
        name: T,
        elem_ty: LLVMTypeRef,
        c_elem_ty: String,
        context: LLVMContextRef,
        module: LLVMModuleRef,
        ccontext: CContextRef,
    ) -> Vector {
        // for C
        let mut def = CodeBuilder::new();
        def.add("typedef struct {");
        def.add(format!("{elem_ty}* data;", elem_ty=c_elem_ty));
        def.add(format!("{u64} size;", u64=c_u64_type(ccontext)));
        def.add(format!("}} {};", name.as_ref()));
        (*ccontext).prelude_code.add(def.result());
        // for LLVM
        let c_name = CString::new(name.as_ref()).unwrap();
        let mut layout = [LLVMPointerType(elem_ty, 0), LLVMInt64TypeInContext(context)];
        let vector = LLVMStructCreateNamed(context, c_name.as_ptr());
        LLVMStructSetBody(vector, layout.as_mut_ptr(), layout.len() as u32, 0);
        Vector {
            name: c_name.into_string().unwrap(),
            context,
            module,
            ccontext,
            vector_ty: vector,
            elem_ty,
            c_elem_ty,
            new: None,
            c_new: String::new(),
            clone: None,
            c_clone: String::new(),
            at: None,
            c_at: String::new(),
            vat: None,
            c_vat: String::new(),
            size: None,
            c_size: String::new(),
            slice: None,
            c_slice: String::new(),
            extend: None,
            c_extend: String::new(),
        }
    }

    /// Generates the `new` method on vectors and calls it.
    ///
    /// The new method allocates a buffer of size exactly `size`. The memory allocated for the
    /// vector is uninitialized.
    pub unsafe fn gen_new(
        &mut self,
        builder: LLVMBuilderRef,
        intrinsics: &mut Intrinsics,
        run: LLVMValueRef,
        size: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.new.is_none() {
            let mut arg_tys = [self.i64_type(), self.run_handle_type()];
            let ret_ty = self.vector_ty;
            let c_arg_tys = [self.c_i64_type(), self.c_run_handle_type()];
            let c_ret_ty = &self.name.clone();

            let name = format!("{}.new", self.name);
            let (function, builder, _, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name, false);

            let size = LLVMGetParam(function, 0);
            let elem_size = self.size_of(self.elem_ty);
            let alloc_size = LLVMBuildMul(builder, elem_size, size, c_str!("size"));
            let run = LLVMGetParam(function, 1);
            let bytes =
                intrinsics.call_weld_run_malloc(builder, run, alloc_size, Some(c_str!("bytes")));
            let elements = LLVMBuildBitCast(
                builder,
                bytes,
                LLVMPointerType(self.elem_ty, 0),
                c_str!("elements"),
            );
            let mut result = LLVMGetUndef(self.vector_ty);
            result = LLVMBuildInsertValue(builder, result, elements, POINTER_INDEX, c_str!(""));
            result = LLVMBuildInsertValue(builder, result, size, SIZE_INDEX, c_str!(""));
            LLVMBuildRet(builder, result);

            self.new = Some(function);
            LLVMDisposeBuilder(builder);
        }

        let mut args = [size, run];
        Ok(LLVMBuildCall(
            builder,
            self.new.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }

    /// Generates the `clone` method on vectors and calls it.
    ///
    /// The clone method performs a shallow copy of the vector.
    pub unsafe fn gen_clone(
        &mut self,
        builder: LLVMBuilderRef,
        intrinsics: &mut Intrinsics,
        vector: LLVMValueRef,
        run: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.clone.is_none() {
            let mut arg_tys = [self.vector_ty, self.run_handle_type()];
            let ret_ty = self.vector_ty;
            let c_arg_tys = [self.name.clone(), self.c_run_handle_type()];
            let c_ret_ty = &self.name.clone();

            let name = format!("{}.clone", self.name);
            let (function, builder, _, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name, false);

            let vector = LLVMGetParam(function, 0);
            let run = LLVMGetParam(function, 1);

            let elem_size = self.size_of(self.elem_ty);
            let size = LLVMBuildExtractValue(builder, vector, SIZE_INDEX, c_str!(""));
            let alloc_size = LLVMBuildMul(builder, elem_size, size, c_str!("size"));

            let dst_bytes =
                intrinsics.call_weld_run_malloc(builder, run, alloc_size, Some(c_str!("")));
            let source_bytes = LLVMBuildExtractValue(builder, vector, POINTER_INDEX, c_str!(""));
            let source_bytes =
                LLVMBuildBitCast(builder, source_bytes, self.void_pointer_type(), c_str!(""));
            let _ = intrinsics.call_memcpy(builder, dst_bytes, source_bytes, alloc_size);

            let elements = LLVMBuildBitCast(
                builder,
                dst_bytes,
                LLVMPointerType(self.elem_ty, 0),
                c_str!(""),
            );
            let result = LLVMBuildInsertValue(
                builder,
                LLVMGetUndef(self.vector_ty),
                elements,
                POINTER_INDEX,
                c_str!(""),
            );
            let result = LLVMBuildInsertValue(builder, result, size, SIZE_INDEX, c_str!(""));
            LLVMBuildRet(builder, result);

            self.clone = Some(function);
            LLVMDisposeBuilder(builder);
        }

        let mut args = [vector, run];
        Ok(LLVMBuildCall(
            builder,
            self.clone.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }

    /// Generates the `at` method on vectors.
    ///
    /// This method performs an index computation into the vector.  The function returns a pointer
    /// to the requested index: it does not dereference the pointer.
    pub unsafe fn define_at(
        &mut self,
    ) {
        if self.at.is_none() {
            let mut arg_tys = [self.vector_ty, self.i64_type()];
            let ret_ty = LLVMPointerType(self.elem_ty, 0);
            let c_arg_tys = [
                // Use pointer of vector insteaf of vector.
                self.c_pointer_type(&self.name),
                self.c_u64_type(),
            ];
            let c_ret_ty = &self.c_pointer_type(&self.c_elem_ty);

            let name = format!("{}_at", self.name);
            let (function, builder, _, mut c_code) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name.clone(), true);

            // for C
            c_code.add("{");
            c_code.add(format!(
                "return &{}->data[{}];",
                self.c_get_param(0),
                self.c_get_param(1),
            ));
            c_code.add("}");
            (*self.ccontext()).prelude_code.add(c_code.result());
            self.c_at = name;

            // for LLVM
            LLVMExtAddAttrsOnFunction(self.context, function, &[AlwaysInline]);

            let vector = LLVMGetParam(function, 0);
            let index = LLVMGetParam(function, 1);
            let pointer = LLVMBuildExtractValue(builder, vector, POINTER_INDEX, c_str!(""));
            let value_pointer = LLVMBuildGEP(builder, pointer, [index].as_mut_ptr(), 1, c_str!(""));
            LLVMBuildRet(builder, value_pointer);

            self.at = Some(function);
            LLVMDisposeBuilder(builder);
        }
    }
    /// Generates the `at` method on vectors and calls it.
    ///
    /// This method performs an index computation into the vector.  The function returns a pointer
    /// to the requested index: it does not dereference the pointer.
    pub unsafe fn gen_at(
        &mut self,
        builder: LLVMBuilderRef,
        vector: LLVMValueRef,
        index: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.at.is_none() {
            self.define_at();
        }

        let mut args = [vector, index];
        Ok(LLVMBuildCall(
            builder,
            self.at.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }
    pub unsafe fn c_gen_at(
        &mut self,
        _builder: LLVMBuilderRef,
        vector: &str,
        index: &str,
    ) -> WeldResult<String> {
        if self.at.is_none() {
            self.define_at();
        }

        Ok(format!("{}(&{}, {})", self.c_at, vector, index))
    }


    /// Generate the `slice` method on vectors and calls it.
    ///
    /// This method takes an index and size returns returns a view into the given vector. If index
    /// is out of bounds, behavior is undefined. If `index + size` is greater than the length of
    /// the vector, a slice up to the end of the vector starting at index is returned.
    pub unsafe fn gen_slice(
        &mut self,
        builder: LLVMBuilderRef,
        vector: LLVMValueRef,
        index: LLVMValueRef,
        size: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        use self::llvm_sys::LLVMIntPredicate::LLVMIntUGT;
        if self.slice.is_none() {
            let mut arg_tys = [self.vector_ty, self.i64_type(), self.i64_type()];
            let ret_ty = self.vector_ty;
            let c_arg_tys = [self.name.clone(), self.c_i64_type(), self.c_i64_type()];
            let c_ret_ty = &self.name.clone();

            let name = format!("{}.slice", self.name);
            let (function, builder, _, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name, false);

            let vector = LLVMGetParam(function, 0);
            let index = LLVMGetParam(function, 1);
            let size = LLVMGetParam(function, 2);

            // Compute the size of the array. We use the remaining size if the new size does not
            // accomodate the vector starting at the given index.
            let cur_size = LLVMBuildExtractValue(builder, vector, SIZE_INDEX, c_str!(""));
            let remaining = LLVMBuildSub(builder, cur_size, index, c_str!(""));
            let size_cmp = LLVMBuildICmp(builder, LLVMIntUGT, size, remaining, c_str!(""));
            let new_size = LLVMBuildSelect(builder, size_cmp, remaining, size, c_str!(""));

            let elements = LLVMBuildExtractValue(builder, vector, POINTER_INDEX, c_str!(""));
            let new_elements = LLVMBuildGEP(builder, elements, [index].as_mut_ptr(), 1, c_str!(""));

            let mut result = LLVMGetUndef(self.vector_ty);
            result = LLVMBuildInsertValue(builder, result, new_elements, POINTER_INDEX, c_str!(""));
            result = LLVMBuildInsertValue(builder, result, new_size, SIZE_INDEX, c_str!(""));
            LLVMBuildRet(builder, result);

            self.slice = Some(function);
            LLVMDisposeBuilder(builder);
        }

        let mut args = [vector, index, size];
        Ok(LLVMBuildCall(
            builder,
            self.slice.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }

    /// Generates the `vat` method on vectors and calls it.
    ///
    /// This method performs an index computation into the vector.  The function returns a SIMD pointer
    /// to the requested index: it does not dereference the pointer. The generated method does not perform
    /// any bounds checking.
    pub unsafe fn gen_vat(
        &mut self,
        builder: LLVMBuilderRef,
        vector: LLVMValueRef,
        index: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.vat.is_none() {
            let mut arg_tys = [self.vector_ty, self.i64_type()];
            let ret_ty = LLVMPointerType(LLVMVectorType(self.elem_ty, LLVM_VECTOR_WIDTH), 0);
            let c_arg_tys = [self.name.clone(), self.c_i64_type()];
            let c_ret_ty = &self.c_pointer_type(&self.c_simd_type(&self.c_elem_ty, LLVM_VECTOR_WIDTH));

            let name = format!("{}.vat", self.name);
            let (function, builder, _, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name, true);

            LLVMExtAddAttrsOnFunction(self.context, function, &[AlwaysInline]);

            let vector = LLVMGetParam(function, 0);
            let index = LLVMGetParam(function, 1);
            let pointer = LLVMBuildExtractValue(builder, vector, 0, c_str!(""));
            let value_pointer = LLVMBuildGEP(builder, pointer, [index].as_mut_ptr(), 1, c_str!(""));
            let value_pointer = LLVMBuildBitCast(builder, value_pointer, ret_ty, c_str!(""));
            LLVMBuildRet(builder, value_pointer);

            self.vat = Some(function);
            LLVMDisposeBuilder(builder);
        }

        let mut args = [vector, index];
        Ok(LLVMBuildCall(
            builder,
            self.vat.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }

    /// Helper function to generate the `size` method on vectors.
    pub unsafe fn define_size(
        &mut self,
    ) {
        if self.size.is_none() {
            // Generate size function only once.
            let mut arg_tys = [self.vector_ty];
            let ret_ty = self.u64_type();
            let c_arg_tys = [
                // Use pointer of vector insteaf of vector.
                self.c_pointer_type(&self.name),
            ];
            let c_ret_ty = &self.c_u64_type();

            // Use C name.
            let name = format!("{}_size", self.name);
            let (function, builder, _, mut c_code) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name.clone(), true);

            // for C
            c_code.add("{");
            c_code.add(format!("return {}->size;", self.c_get_param(0)));
            c_code.add("}");
            (*self.ccontext()).prelude_code.add(c_code.result());
            self.c_size = name;
            // for LLVM
            LLVMExtAddAttrsOnFunction(self.context, function, &[AlwaysInline]);

            let vector = LLVMGetParam(function, 0);
            let size = LLVMBuildExtractValue(builder, vector, SIZE_INDEX, c_str!(""));
            LLVMBuildRet(builder, size);

            self.size = Some(function);
            LLVMDisposeBuilder(builder);
        }
    }
    /// Generates the `size` method on vectors and calls it.
    ///
    /// This returns the size (equivalently, the capacity) of the vector.
    pub unsafe fn gen_size(
        &mut self,
        builder: LLVMBuilderRef,
        vector: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        if self.size.is_none() {
            self.define_size();
        }

        let mut args = [vector];
        Ok(LLVMBuildCall(
            builder,
            self.size.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }
    pub unsafe fn c_gen_size(
        &mut self,
        _builder: LLVMBuilderRef,
        vector: &str,
    ) -> WeldResult<String> {
        if self.size.is_none() {
            // define both C and LLVM functions.
            self.define_size();
        }

        Ok(format!("{}(&{})", self.c_size, vector))
    }

    /// Generates the `extend` method on vectors and calls it.
    ///
    /// This method grows the capacity of vector to exactly `size` and returns a new vector. If
    /// the input vector can already accomodate `size` elements, the same vector is returned
    /// unmodified.
    ///
    /// This method modifies the size to be the new capacity if the vector is resized.
    pub unsafe fn gen_extend(
        &mut self,
        builder: LLVMBuilderRef,
        intrinsics: &mut Intrinsics,
        run: LLVMValueRef,
        vector: LLVMValueRef,
        size: LLVMValueRef,
    ) -> WeldResult<LLVMValueRef> {
        use self::llvm_sys::LLVMIntPredicate::LLVMIntSGT;
        if self.extend.is_none() {
            trace!("Generated extend");
            let mut arg_tys = [self.vector_ty, self.i64_type(), self.run_handle_type()];
            let ret_ty = self.vector_ty;
            let c_arg_tys = [self.name.clone(), self.c_i64_type(), self.c_run_handle_type()];
            let c_ret_ty = &self.name.clone();

            let name = format!("{}.extend", self.name);
            let (function, builder, entry_block, _) = self.define_function(ret_ty, c_ret_ty, &mut arg_tys, &c_arg_tys, name, false);

            let realloc_block = LLVMAppendBasicBlockInContext(self.context, function, c_str!(""));
            let finish_block = LLVMAppendBasicBlockInContext(self.context, function, c_str!(""));

            let vector = LLVMGetParam(function, 0);
            let requested_size = LLVMGetParam(function, 1);
            let run_handle = LLVMGetParam(function, 2);

            let current_size = LLVMBuildExtractValue(builder, vector, SIZE_INDEX, c_str!(""));

            let resize_flag = LLVMBuildICmp(
                builder,
                LLVMIntSGT,
                requested_size,
                current_size,
                c_str!(""),
            );
            LLVMBuildCondBr(builder, resize_flag, realloc_block, finish_block);
            trace!("finished entry block");

            // Build block where memory is grown to accomdate the requested size.
            LLVMPositionBuilderAtEnd(builder, realloc_block);
            let pointer = LLVMBuildExtractValue(builder, vector, POINTER_INDEX, c_str!(""));
            let alloc_size = LLVMBuildNSWMul(
                builder,
                requested_size,
                self.size_of(self.elem_ty),
                c_str!(""),
            );
            let raw_pointer = LLVMBuildBitCast(
                builder,
                pointer,
                LLVMPointerType(self.i8_type(), 0),
                c_str!(""),
            );
            let bytes = intrinsics.call_weld_run_realloc(
                builder,
                run_handle,
                raw_pointer,
                alloc_size,
                Some(c_str!("")),
            );
            let resized_elements =
                LLVMBuildBitCast(builder, bytes, LLVMTypeOf(pointer), c_str!(""));

            let resized = LLVMBuildInsertValue(
                builder,
                LLVMGetUndef(self.vector_ty),
                resized_elements,
                POINTER_INDEX,
                c_str!(""),
            );
            let resized =
                LLVMBuildInsertValue(builder, resized, requested_size, SIZE_INDEX, c_str!(""));
            LLVMBuildBr(builder, finish_block);
            trace!("finished reallocation block");

            LLVMPositionBuilderAtEnd(builder, finish_block);
            let return_value = LLVMBuildPhi(builder, self.vector_ty, c_str!(""));
            let mut values = [vector, resized];
            let mut blocks = [entry_block, realloc_block];
            LLVMAddIncoming(
                return_value,
                values.as_mut_ptr(),
                blocks.as_mut_ptr(),
                values.len() as u32,
            );
            LLVMBuildRet(builder, return_value);
            trace!("finished extend");

            self.extend = Some(function);
            LLVMDisposeBuilder(builder);
        }

        let mut args = [vector, size, run];
        Ok(LLVMBuildCall(
            builder,
            self.extend.unwrap(),
            args.as_mut_ptr(),
            args.len() as u32,
            c_str!(""),
        ))
    }
}
