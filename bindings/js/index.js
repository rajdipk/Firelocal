const platform = process.platform;
const arch = process.arch;

let nativeBinding;
let binaryName;

if (platform === 'win32') {
    binaryName = `firelocal-js.win32-${arch}-msvc.node`;
} else if (platform === 'darwin') {
    binaryName = `firelocal-js.darwin-${arch}.node`;
} else if (platform === 'linux') {
    binaryName = `firelocal-js.linux.node`;
} else {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
}

try {
    nativeBinding = require(`./${binaryName}`);
    module.exports = nativeBinding;
} catch (err) {
    throw new Error(`Failed to load FireLocal native binding for ${platform}-${arch}: ${err.message}`);
}
