; ModuleID = 'chinese_program'
source_filename = "chinese_program"

@"\E7\AC\AC\E4\B8\80\E5\8F\A5" = private unnamed_addr constant [16 x i8] c"\E9\9B\B6\E6\88\90\E6\9C\AC\E6\8A\BD\E8\B1\A1\00", align 1
@"\E7\AC\AC\E4\BA\8C\E5\8F\A5" = private unnamed_addr constant [26 x i8] c"\E6\80\A7\E8\83\BD\E6\97\A0\E5\8F\8C\E7\9A\84Rust\E8\AF\AD\E8\A8\80\00", align 1
@concat_fmt = private unnamed_addr constant [8 x i8] c"%s, %s.\00", align 1
@fmt = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@"\E7\AC\AC\E4\B8\89\E5\8F\A5" = private unnamed_addr constant [16 x i8] c"\E6\97\A0\E5\9E\83\E5\9C\BE\E5\9B\9E\E6\94\B6\00", align 1
@"\E7\AC\AC\E5\9B\9B\E5\8F\A5" = private unnamed_addr constant [19 x i8] c"\E5\8D\B4\E6\97\A0\E5\86\85\E5\AD\98\E6\B3\84\E6\BC\8F\00", align 1
@"\E7\AC\AC\E4\BA\94\E5\8F\A5" = private unnamed_addr constant [31 x i8] c"\E4\BB\8E\E7\B3\BB\E7\BB\9F\E5\BA\95\E5\B1\82\E5\88\B0\E7\BD\91\E7\BB\9C\E6\9C\8D\E5\8A\A1\00", align 1
@"\E7\AC\AC\E5\85\AD\E5\8F\A5" = private unnamed_addr constant [11 x i8] c"Rust\E8\AF\AD\E8\A8\80\00", align 1
@"\E7\AC\AC\E4\B8\83\E5\8F\A5" = private unnamed_addr constant [28 x i8] c"\E7\8E\B0\E4\BB\A3\E7\B3\BB\E7\BB\9F\E7\BC\96\E7\A8\8B\E7\9A\84\E6\9C\AA\E6\9D\A5\00", align 1
@concat_fmt.1 = private unnamed_addr constant [20 x i8] c"%s, %s, %s, %s, %s.\00", align 1
@fmt.2 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

declare i32 @printf(ptr, ...)

declare i32 @sprintf(ptr, ptr, ...)

define i32 @main() {
entry:
  %concat_buffer = alloca i8, i8 0, align 1
  %sprintf_call = call i32 (ptr, ptr, ...) @sprintf(ptr %concat_buffer, ptr @concat_fmt, ptr @"\E7\AC\AC\E4\B8\80\E5\8F\A5", ptr @"\E7\AC\AC\E4\BA\8C\E5\8F\A5")
  %printf_call = call i32 (ptr, ...) @printf(ptr @fmt, ptr %concat_buffer)
  %concat_buffer1 = alloca i8, i8 0, align 1
  %sprintf_call2 = call i32 (ptr, ptr, ...) @sprintf(ptr %concat_buffer1, ptr @concat_fmt.1, ptr @"\E7\AC\AC\E4\B8\89\E5\8F\A5", ptr @"\E7\AC\AC\E5\9B\9B\E5\8F\A5", ptr @"\E7\AC\AC\E4\BA\94\E5\8F\A5", ptr @"\E7\AC\AC\E5\85\AD\E5\8F\A5", ptr @"\E7\AC\AC\E4\B8\83\E5\8F\A5")
  %printf_call3 = call i32 (ptr, ...) @printf(ptr @fmt.2, ptr %concat_buffer1)
  ret i32 0
}
