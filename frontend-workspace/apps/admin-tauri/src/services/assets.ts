// ────────────────────────────────────────────────────────────────
// assets.ts — OSS image upload + local asset resolution
// assets.ts — OSS 图片上传 + 本地资源解析
// ────────────────────────────────────────────────────────────────

import * as bridge from "@/bridge/invoke";
import { assetUrl } from "@/bridge/protocol";

const MAX_IMAGE_BYTES = 10 * 1024 * 1024; // 10 MB

export async function uploadImage(
  blob: Blob,
  filename: string,
): Promise<string> {
  if (blob.size > MAX_IMAGE_BYTES) {
    throw new Error(
      `Image too large: ${(blob.size / 1024 / 1024).toFixed(1)} MB`,
    );
  }

  const buffer = new Uint8Array(await blob.arrayBuffer());
  return bridge.uploadImage(buffer, filename);
}

/**
 * Resolve asset URL: ms:// protocol first (local cache), fallback to cloud URL.
 */
export function resolveAssetUrl(
  localPath: string,
  cloudUrl?: string | null,
): string {
  if (localPath) return assetUrl(localPath);
  if (cloudUrl) return cloudUrl;
  return "";
}
