from hatchling.builders.hooks.plugin.interface import BuildHookInterface
import platform

class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        """
        Sets force-include option for glsdk/libglsdk shared lib.

        Does the same as [tool.hatch.build.targets.wheel.force-include], however
        assumes system OS and based on that sets the correct extension.
        """
        system = platform.system()

        match system:
            case "Darwin":
                lib_ext = ".dylib"
            case "Windows":
                lib_ext = ".dll"
            case _:
                lib_ext = ".so"

        shared_file = f"glsdk/libglsdk{lib_ext}"
        build_data['force_include'][shared_file] = shared_file