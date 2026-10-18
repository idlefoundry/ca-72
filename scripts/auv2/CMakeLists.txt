# The CA-72 as an Audio Unit (version 2) for macOS: clap-wrapper's AUv2 wrapper around the
# CLAP bundle, which goes inside it, in Contents/PlugIns, so that the component plays the
# CLAP it was built with whether or not a CLAP is installed (decisions.md R30). Built by
# scripts/auv2.sh, which passes:
#
#   CA72_CLAP           the CLAP bundle to wrap (target/bundled/CA-72.clap)
#   CA72_VERSION        the plug-in's version (Cargo.toml's)
#   CLAP_WRAPPER_ROOT   clap-wrapper's sources (third_party/clap-wrapper)
#   CLAP_SDK_ROOT       the CLAP headers' (third_party/clap)
#   AUDIOUNIT_SDK_ROOT  Apple's AudioUnitSDK's (third_party/AudioUnitSDK)
cmake_minimum_required(VERSION 3.21)

# As the bundles' Intel slice (.cargo/config.toml): macOS 11, for the audio workgroups.
set(CMAKE_OSX_DEPLOYMENT_TARGET 11.0 CACHE STRING "")
set(CMAKE_CXX_STANDARD 17)

project(ca72-auv2 VERSION ${CA72_VERSION} LANGUAGES C CXX OBJC OBJCXX)

foreach(v CA72_CLAP CLAP_WRAPPER_ROOT CLAP_SDK_ROOT AUDIOUNIT_SDK_ROOT)
    if(NOT DEFINED ${v})
        message(FATAL_ERROR "ca72-auv2: ${v} is not set (scripts/auv2.sh sets it)")
    endif()
endforeach()

add_subdirectory(${CLAP_WRAPPER_ROOT} clap-wrapper EXCLUDE_FROM_ALL)

# The component's codes are its identity in every host's saved songs: never to change.
# "aumu", an instrument; the manufacturer's code has capitals, as Apple asks of everyone but
# itself.
add_library(ca72-auv2 MODULE)
target_add_auv2_wrapper(
    TARGET ca72-auv2
    OUTPUT_NAME "CA-72"
    BUNDLE_IDENTIFIER "com.idlefoundry.ca-72.component"
    BUNDLE_VERSION "${CA72_VERSION}"
    MANUFACTURER_NAME "Idle Foundry"
    MANUFACTURER_CODE "IdlF"
    SUBTYPE_CODE "CA72"
    INSTRUMENT_TYPE "aumu"
    MACOS_EMBEDDED_CLAP_LOCATION "${CA72_CLAP}"
)

# Exercise the actual AU event adapter: no editor or physical audio/MIDI device.
enable_testing()
add_executable(ca72-native-occlusion-tests tests/occlusion.cpp)
target_link_libraries(ca72-native-occlusion-tests PRIVATE ca72-auv2-clap-wrapper-auv2-lib
    "-framework Foundation" "-framework CoreFoundation"
    "-framework AudioToolbox" "-framework CoreMIDI")
add_test(NAME native-occlusion-capability COMMAND ca72-native-occlusion-tests)

add_executable(ca72-au-midi-tests
    tests/midi.cpp
    ${CLAP_WRAPPER_ROOT}/src/detail/auv2/process.cpp
    ${CLAP_WRAPPER_ROOT}/src/detail/auv2/parameter.cpp
)
target_link_libraries(ca72-au-midi-tests PRIVATE ca72-auv2-clap-wrapper-auv2-lib
    "-framework Foundation" "-framework CoreFoundation"
    "-framework AudioToolbox" "-framework CoreMIDI")
add_test(NAME au-midi-adapter COMMAND ca72-au-midi-tests)

add_executable(ca72-au-render-tests tests/render.mm)
target_compile_options(ca72-au-render-tests PRIVATE -fobjc-arc)
target_link_libraries(ca72-au-render-tests PRIVATE "-framework Foundation"
    "-framework AudioToolbox" "-framework CoreMIDI")
add_test(NAME au-midi-render COMMAND ca72-au-render-tests $<TARGET_BUNDLE_DIR:ca72-auv2>)
set_tests_properties(au-midi-render PROPERTIES TIMEOUT 120
    ENVIRONMENT "IDLE_FOUNDRY_SHARED=${CMAKE_CURRENT_BINARY_DIR}/test-user-data")
