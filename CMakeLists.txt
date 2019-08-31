cmake_minimum_required(VERSION 3.5)
cmake_policy(SET CMP0015 NEW)
cmake_policy(SET CMP0022 NEW)

project(cascade)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -pthread -g -Werror -Wall -pedantic -Wextra -fPIC -Wnon-virtual-dtor -std=c++17")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -Wl,-z,defs")

include(FindPkgConfig)
pkg_check_modules(MIRAL miral REQUIRED)
pkg_check_modules(FREETYPE freetype2 REQUIRED)
pkg_check_modules(WAYLAND_CLIENT REQUIRED wayland-client)
pkg_check_modules(XKBCOMMON xkbcommon REQUIRED)
find_package(Boost COMPONENTS filesystem REQUIRED)

add_executable(cascade
    cascade.cpp
    eglauncher.cpp eglauncher.h
    egwallpaper.cpp egwallpaper.h
    egwindowmanager.cpp egwindowmanager.h
    egworker.cpp egworker.h
    printer.cpp printer.h
    egfullscreenclient.cpp egfullscreenclient.h
    ffi_helpers.cpp
    rust_wm/rust_wm.h)

execute_process(
    COMMAND grep ^NAME= /etc/os-release 
    COMMAND cut -d= -f2
    OUTPUT_STRIP_TRAILING_WHITESPACE
    OUTPUT_VARIABLE CASCADE_OS)


if (CASCADE_OS STREQUAL "Ubuntu")
  set(CASCADE_WALLPAPER_BOTTOM 0x92006a)
elseif (CASCADE_OS STREQUAL "Fedora")
  set(CASCADE_WALLPAPER_BOTTOM 0x25487c)
else()
  set(CASCADE_WALLPAPER_BOTTOM 0x92006a)
endif()

set_source_files_properties(cascade.cpp PROPERTIES COMPILE_DEFINITIONS CASCADE_WALLPAPER_BOTTOM="${CASCADE_WALLPAPER_BOTTOM}")

target_include_directories(cascade PUBLIC SYSTEM ${MIRAL_INCLUDE_DIRS} ${Boost_INCLUDE_DIRS} ${FREETYPE_INCLUDE_DIRS})
target_link_libraries(     cascade               ${MIRAL_LDFLAGS}      ${WAYLAND_CLIENT_LIBRARIES}  ${Boost_LIBRARIES}    ${FREETYPE_LIBRARIES})
target_link_libraries(     cascade               ${XKBCOMMON_LIBRARIES})
target_link_libraries(     cascade               ${CMAKE_BINARY_DIR}/../rust_wm/target/debug/librust_wm.so)


add_custom_target(cascade-launch ALL
    cp ${CMAKE_CURRENT_SOURCE_DIR}/cascade-launch.sh ${CMAKE_BINARY_DIR}/cascade-launch
)

install(PROGRAMS ${CMAKE_BINARY_DIR}/cascade-launch ${CMAKE_BINARY_DIR}/cascade
    DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
)

install(FILES ${CMAKE_SOURCE_DIR}/cascade.desktop
    DESTINATION /usr/share/wayland-sessions/
)
