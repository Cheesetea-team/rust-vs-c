cmake_minimum_required(VERSION 3.20)
project(fire)
add_executable(fire main.c)
target_link_libraries(fire raylib)