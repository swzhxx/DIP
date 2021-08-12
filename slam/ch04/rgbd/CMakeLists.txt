# 添加c++ 11标准支持

option(USE_UBUNTU_20 "Set to ON if you are using Ubuntu 20.04" ON)
if(USE_UBUNTU_20)
    message("You are using Ubuntu 20.04, fmt::fmt will be linked")
    find_package(fmt REQUIRED)
    set(FMT_LIBRARIES fmt::fmt)
endif()

find_package(Sophus REQUIRED)
include_directories(${Sophus_INCLUDE_DIRS})
include_directories("/usr/include/eigen3")
find_package(Pangolin REQUIRED)
find_package(OpenCV REQUIRED)

add_executable(joinMap joinMap.cpp)
target_link_libraries(joinMap ${FMT_LIBRARIES} ${OpenCV_LIBS} ${Pangolin_LIBRARIES} )
