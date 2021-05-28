cmake_minimum_required(VERSION 3.13)

project(NAME)

# Add sources to a target from a specified directory
function(add_sources target dir)
    cmake_parse_arguments(PARSE_ARGV 2 ADD "" "" "SRCS")
    list(TRANSFORM ADD_SRCS PREPEND ${dir})
    target_sources(${target} PRIVATE ${ADD_SRCS})
    target_include_directories(${target} PRIVATE ${dir})
endfunction()

# Find the requested library
set(GlkLibPath "glk" CACHE PATH "Glk Library Path")
set(GlkLibName "" CACHE STRING "Glk Library Name (without lib- or -.a)")
if (GlkLibName STREQUAL "")
    get_filename_component(GlkLibNameReal ${GlkLibPath} NAME)
else ()
    set(GlkLibNameReal ${GlkLibName} STRING)
endif()
add_library(glk STATIC IMPORTED)
set_target_properties(glk PROPERTIES IMPORTED_LOCATION "${GlkLibPath}/lib${GlkLibNameReal}.a")
target_include_directories(glk INTERFACE ${GlkLibPath})

# Prepare the image data as a library
add_library(image STATIC image.o)
set_target_properties(image PROPERTIES LINKER_LANGUAGE C)
add_custom_command(OUTPUT image.o
    COMMAND cd ${CMAKE_CURRENT_SOURCE_DIR} && ld -r -b binary -o ${CMAKE_CURRENT_BINARY_DIR}/image.o image.data
    COMMAND objcopy --rename-section .data=.rodata,alloc,load,readonly,data,contents ${CMAKE_CURRENT_BINARY_DIR}/image.o ${CMAKE_CURRENT_BINARY_DIR}/image.o)
set_source_files_properties(image.o PROPERTIES EXTERNAL_OBJECT true GENERATED true)

# And now our project
add_executable(EXENAME)
target_link_libraries(EXENAME glk image m)
add_sources(EXENAME "glulxe/" SRCS GLULXE_FILES)
target_include_directories(EXENAME PRIVATE glulxe/)
add_sources(EXENAME "./"
    SRCS functions_safe.c functions_unsafe.c runtime.c unixstrt.c)
target_compile_definitions(EXENAME PRIVATE OS_UNIX FLOAT_COMPILE_SAFER_POWF)
target_compile_options(EXENAME PRIVATE -Wall -Wmissing-prototypes
    -Wstrict-prototypes -Wno-overflow -Wno-unused)
if (CMAKE_C_COMPILER_ID MATCHES "Clang")
    target_compile_options(EXENAME PRIVATE
        -fbracket-depth=5000
        -Wno-constant-conversion -Wno-integer-overflow)
    set_source_files_properties(functions_safe.c functions_unsafe.c
        PROPERTIES COMPILE_OPTIONS "-Wconditional-uninitialized")
endif()