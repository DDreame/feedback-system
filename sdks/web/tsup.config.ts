import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  minify: false,
  target: ['es2020', 'chrome80', 'firefox80', 'safari14'],
  outDir: 'dist',
  name: 'FeedbackWidget',
  globals: {
    'index': 'FeedbackWidget',
  },
});
