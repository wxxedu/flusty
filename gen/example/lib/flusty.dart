
import 'dart:ffi' as ffi;
import 'dart:io' show Platform, Directory;
import 'package:path/path.dart' as path;

typedef hello_world = ffi.Void Function();
typedef HelloWorld = void Function();
final class Flusty {
    static ffi.DynamicLibrary? _dylib;

    static ffi.DynamicLibrary dylib() {
        if (_dylib != null) {
            return _dylib!;
        }
        var libraryPath = path.join(
            Directory.current.path, 
            'native', 'target', 'release', 
            'libnative.so'
        );
        if (Platform.isMacOS) {
            libraryPath = path.join(
                Directory.current.path, 
                'native', 'target', 'release', 
                'libnative.dylib'
            );
        } else if (Platform.isWindows) {
            libraryPath = path.join(
                Directory.current.path, 
                'native', 'target', 'release', 
                'libnative.dll'
            );
        }
        _dylib = ffi.DynamicLibrary.open(libraryPath);
        return _dylib!;
    }

final HelloWorld helloWorld = dylib().lookup<ffi.NativeFunction<hello_world>>('hello_world').asFunction();
}
