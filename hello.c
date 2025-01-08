/**
 * LLVM equivalent of:
 *
 * int sum(int a, int b) {
 *     return a + b;
 * }
 */

#include <llvm-c/Core.h>
#include <llvm-c/Target.h>
#include <llvm-c/Analysis.h>
#include <llvm-c/BitWriter.h>
#include <llvm-c/Comdat.h>

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char const *argv[]) {
    LLVMModuleRef mod = LLVMModuleCreateWithName("my_module");

    LLVMBuilderRef builder = LLVMCreateBuilder();

    // Sum function
    LLVMTypeRef param_types[] = { LLVMInt32Type(), LLVMInt32Type() };
    LLVMTypeRef sum_ret_type = LLVMFunctionType(LLVMInt32Type(), param_types, 2, 0);

    {
        
        
        LLVMValueRef sum = LLVMAddFunction(mod, "sum", sum_ret_type);
        LLVMBasicBlockRef entry = LLVMAppendBasicBlock(sum, "entry");
        LLVMPositionBuilderAtEnd(builder, entry);
        LLVMValueRef tmp = LLVMBuildAdd(builder, LLVMGetParam(sum, 0), LLVMGetParam(sum, 1), "tmp");
        LLVMBuildRet(builder, tmp);


        LLVMContextRef context = LLVMGetModuleContext(mod);
        LLVMAttributeRef noundef_attr = LLVMCreateEnumAttribute(context, LLVMGetEnumAttributeKindForName("noundef", 7), 0);
        LLVMAddAttributeAtIndex(sum, 1, noundef_attr); // First parameter
        LLVMAddAttributeAtIndex(sum, 2, noundef_attr);
    }
    {
        // Main function
        LLVMTypeRef ret_type = LLVMFunctionType(LLVMInt32Type(), NULL, 0, 0);
        LLVMValueRef main = LLVMAddFunction(mod, "main", ret_type);
        LLVMBasicBlockRef entry = LLVMAppendBasicBlock(main, "entry");
        LLVMPositionBuilderAtEnd(builder, entry);
        LLVMValueRef sum = LLVMBuildCall2(builder, sum_ret_type, LLVMGetNamedFunction(mod, "sum"), (LLVMValueRef[]){ LLVMConstInt(LLVMInt32Type(), 1, 0), LLVMConstInt(LLVMInt32Type(), 2, 0) }, 2, "sum");
        LLVMBuildRet(builder, sum);
    }

    LLVMTypeRef printf_param_types[] = { LLVMPointerType(LLVMInt8Type(), 0), LLVMInt32Type() }; // format string + int
    LLVMTypeRef printf_ret_type = LLVMFunctionType(LLVMInt32Type(), printf_param_types, 2, 1);  // returns int, has 2 parameters
    LLVMAddFunction(mod, "printf", printf_ret_type);
    

    char *error = NULL;
    LLVMVerifyModule(mod, LLVMAbortProcessAction, &error);
    LLVMDisposeMessage(error);

    LLVMPrintModuleToFile(mod, "sum.ll", NULL);
    // Write out bitcode to file
    if (LLVMWriteBitcodeToFile(mod, "sum.bc") != 0) {
        fprintf(stderr, "error writing bitcode to file, skipping\n");
    }



    LLVMDisposeBuilder(builder);

}