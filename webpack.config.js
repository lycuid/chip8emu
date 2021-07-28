const Package = require("./package.json");
const { resolve } = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const SRC_DIR = resolve(__dirname, "web");
const PUBLIC_DIR = resolve(__dirname, "public");

const CDN_URI = (mode) =>
  mode == "production" ? `${Package.configs.cdn}/p/${Package.name}` : "";

module.exports = (_, { mode }) => ({
  entry: resolve(SRC_DIR, "main.ts"),
  output: {
    filename: "[name].[contenthash].js",
    path: PUBLIC_DIR,
  },
  module: {
    rules: [
      {
        test: /\.tsx?/,
        exclude: /node_modules/,
        loader: "ts-loader",
      },
    ],
  },
  experiments: { asyncWebAssembly: true },
  plugins: [
    new HtmlWebpackPlugin({
      cdn_uri: CDN_URI(mode),
      filename: resolve(PUBLIC_DIR, "index.html"),
      template: resolve(SRC_DIR, "index.html"),
      publicPath: CDN_URI(mode),
    }),
    new WasmPackPlugin({ crateDirectory: __dirname }),
  ],
});
