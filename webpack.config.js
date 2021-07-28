const Package = require("./package.json");
const { resolve } = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");

const SRC_DIR = resolve(__dirname, "web");
const PUBLIC_DIR = resolve(__dirname, "public");

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
      filename: resolve(PUBLIC_DIR, "index.html"),
      template: resolve(SRC_DIR, "index.html"),
      publicPath:
        mode == "production" ? `https://cdn.lycuid.dev/p/${Package.name}` : "",
    }),
    new WasmPackPlugin({ crateDirectory: __dirname, outDir: "pkg" }),
    new CopyWebpackPlugin({
      patterns: [
        { from: resolve(SRC_DIR, "background.svg") },
        { from: resolve(SRC_DIR, "roms", "*") },
      ],
    }),
  ],
});
