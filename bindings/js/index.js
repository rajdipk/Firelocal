try {
    const nativeBinding = require('./index.win32-x64-msvc.node')
    module.exports = nativeBinding
} catch (err) {
    throw err
}
