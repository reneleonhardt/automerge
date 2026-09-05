const path = require("path");
const nodeExternals = require("webpack-node-externals");

// the most basic webpack config for node or web targets for automerge-wasm

const serverConfig = {
  // basic setup for bundling a node package
  target: "node",
  externals: [nodeExternals()],
  externalsPresets: { node: true },

  entry: "./src/index.js",
  output: {
    filename: "node.js",
    path: path.resolve(__dirname, "dist"),
  },
  mode: "development", // or production
};

const clientConfig = {
  target: "web",
  experiments: { asyncWebAssembly: true },
  resolve: {
    alias: {
      "automerge-wasm$": path.resolve(__dirname, "../../web/index.js"),
    },
  },
  entry: "./src/index.js",
  output: {
    filename: "main.js",
    path: path.resolve(__dirname, "public"),
  },
  mode: "development", // or production
  performance: {
    // Keep the generated WASM blob from producing size warnings.
    hints: false,
    maxEntrypointSize: 512000,
    maxAssetSize: 512000,
  },
};

module.exports = [serverConfig, clientConfig];
