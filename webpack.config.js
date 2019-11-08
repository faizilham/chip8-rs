const path = require('path');

const HtmlWebpackPlugin = require("html-webpack-plugin");
const MiniCSSExtractPlugin = require("mini-css-extract-plugin");
const TerserJSPlugin = require('terser-webpack-plugin');
const OptimizeCSSAssetsPlugin = require("optimize-css-assets-webpack-plugin");

const production = process.env.NODE_ENV == "production";

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: production ? "index-[hash:8].js" : "index.js",
  },

  resolve: {
    alias: {
      "wasm-pkg": path.resolve(__dirname, 'pkg')
    }
  },

  mode: production ? "production" : "development",

  optimization: {
    minimizer: [
      new TerserJSPlugin({}),
      new OptimizeCSSAssetsPlugin({})
    ]
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: 'index.html',
    }),
    new MiniCSSExtractPlugin({
      filename: production ? "style-[hash:8].css" : "style.css"
    })
  ],

  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          MiniCSSExtractPlugin.loader,
          "css-loader"
        ]
      }
    ]
  }
};
