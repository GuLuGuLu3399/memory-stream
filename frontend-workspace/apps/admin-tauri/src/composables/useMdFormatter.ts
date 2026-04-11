/**
 * useMdFormatter — V8 格式化引擎
 *
 * 纯前端 Prettier Standalone，毫秒级内存运算。
 * 严格使用 standalone 路径，避免 Node.js 核心模块缺失。
 */
import * as prettier from "prettier/standalone";
import markdownPlugin from "prettier/plugins/markdown";

export async function formatMarkdown(rawText: string): Promise<string> {
  try {
    const result = await prettier.format(rawText, {
      parser: "markdown",
      plugins: [markdownPlugin],
      proseWrap: "preserve",
      tabWidth: 2,
      singleQuote: true,
    });
    return result;
  } catch (err) {
    console.warn("[useMdFormatter] 格式化失败，原样返回:", err);
    return rawText;
  }
}
