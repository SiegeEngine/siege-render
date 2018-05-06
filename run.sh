#!/bin/sh

SDKVER=1.0.68.0
if [ -d "/z/eob/game/client/VulkanSDK/${SDKVER}" ] ; then
    SDK="/z/eob/game/client/VulkanSDK/${SDKVER}"
    export LD_LIBRARY_PATH=${SDK}/x86_64/lib
    export VK_LAYER_PATH=${SDK}/x86_64/etc/explicit_layer.d
elif [ -d "/home/mike/1.PROJECTS/eob/engine/VulkanSDK/${SDKVER}" ] ; then
    SDK="/home/mike/1.PROJECTS/eob/engine/VulkanSDK/${SDKVER}"
    export LD_LIBRARY_PATH=${SDK}/x86_64/lib
    export VK_LAYER_PATH=${SDK}/x86_64/etc/explicit_layer.d
elif [ -d "/c/VulkanSDK/${SDKVER}" ] ; then
    SDK="/c/VulkanSDK/${SDKVER}"
    export VK_LAYER_PATH=${SDK}/Bin
else
    echo "SDK not found"
    exit 1
fi

cargo test || exit 1

# Enable layers here, if desired:
# export VK_INSTANCE_LAYERS="VK_LAYER_GOOGLE_threading:VK_LAYER_LUNARG_parameter_validation:VK_LAYER_LUNARG_object_tracker:VK_LAYER_LUNARG_core_validation:VK_LAYER_LUNARG_swapchain:VK_LAYER_GOOGLE_unique_objects"
# export VK_INSTANCE_LAYERS=VK_LAYER_LUNARG_standard_validation

RUST_BACKTRACE=1 target/debug/examples/colortest
