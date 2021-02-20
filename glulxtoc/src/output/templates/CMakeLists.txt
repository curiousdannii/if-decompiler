cmake_minimum_required(VERSION 3.13)

project(NAME)

# Add sources to a target from a specified directory
function(add_sources target dir)
    cmake_parse_arguments(PARSE_ARGV 2 ADD "" "" "SRCS")
    list(TRANSFORM ADD_SRCS PREPEND ${dir})
    target_sources(${target} PRIVATE ${ADD_SRCS})
    target_include_directories(${target} PRIVATE ${dir})
endfunction()

# Cheapglk library
add_library(cheapglk STATIC)
add_sources(cheapglk "WORKSPACE/upstream/cheapglk/"
    SRCS cgblorb.c cgdate.c cgfref.c cggestal.c cgmisc.c cgschan.c cgstream.c
    cgstyle.c cgunicod.c cgwindow.c gi_blorb.c gi_debug.c gi_dispa.c main.c)
target_compile_options(cheapglk PRIVATE -g -Wall)
target_include_directories(cheapglk PUBLIC WORKSPACE/upstream/cheapglk/)

# Turn Glulxe into a library too
add_library(glulxe STATIC)
target_link_libraries(glulxe cheapglk m)
add_sources(glulxe "WORKSPACE/upstream/glulxe/"
    SRCS accel.c files.c float.c funcs.c gestalt.c glkop.c heap.c main.c
    operand.c osdepend.c profile.c search.c serial.c string.c vm.c)
target_compile_definitions(glulxe PRIVATE OS_UNIX)
target_compile_options(glulxe PRIVATE -Wall -Wmissing-prototypes -Wstrict-prototypes -Wno-unused)
target_include_directories(glulxe PUBLIC WORKSPACE/upstream/glulxe/)

# And now our project
add_executable(NAME)
target_link_libraries(NAME cheapglk glulxe)
add_sources(NAME "OUTDIR/"
    SRCS exec.c functions_safe.c functions_unsafe.c image.c unixstrt.c)