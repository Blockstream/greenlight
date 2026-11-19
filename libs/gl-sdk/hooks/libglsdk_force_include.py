from hatchling.builders.hooks.plugin.interface import BuildHookInterface
import platform

class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        """
        Sets force-include option for glsdk/libglsdk shared lib and the
        generated glsdk/glsdk.py bindings.

        Does the same as [tool.hatch.build.targets.wheel.force-include], however
        assumes system OS and based on that sets the correct extension.

        Both files are listed in .gitignore as generated artifacts, and
        hatchling leaves ignored files out of the wheel unless they are
        force-included.
        """
        build_data['force_include']["glsdk/glsdk.py"] = "glsdk/glsdk.py"

        system = platform.system()

        # Must match the file name `_uniffi_load_indirect` in glsdk.py
        # loads, which on Windows has no `lib` prefix.
        match system:
            case "Darwin":
                lib_name = "libglsdk.dylib"
            case "Windows":
                lib_name = "glsdk.dll"
            case _:
                lib_name = "libglsdk.so"

        shared_file = f"glsdk/{lib_name}"
        build_data['force_include'][shared_file] = shared_file