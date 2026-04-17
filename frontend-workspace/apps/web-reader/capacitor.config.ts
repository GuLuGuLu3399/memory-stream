import type { CapacitorConfig } from "@capacitor/cli";

const config: CapacitorConfig = {
  appId: "com.memorystream.webreader",
  appName: "Memory Stream",
  webDir: "dist",
  server: {
    // 使用 http scheme，避免 https→http 混合内容拦截
    androidScheme: "http",
    // 如果需要调试时连接开发服务器，取消下面的注释：
    // url: "http://192.168.x.x:5173",
  },
  plugins: {
    SplashScreen: {
      launchShowDuration: 1500,
      launchAutoHide: true,
      backgroundColor: "#0a0a0f",
      showSpinner: false,
    },
  },
};

export default config;
