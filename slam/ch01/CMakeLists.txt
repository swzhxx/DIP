project(helloSLAM)

include_directories("include")

#only source files
add_library(libHello src/Hello.cpp)

add_executable( sayHello main.cpp)

target_link_libraries( sayHello libHello )