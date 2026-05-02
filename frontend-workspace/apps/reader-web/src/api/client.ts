// ────────────────────────────────────────────────────────────────
// client.ts — REST API client with response unwrapping + auth retry
// ────────────────────────────────────────────────────────────────

interface ApiResponse<T> {
  code: number;
  message: string;
  data: T;
}

function normalizeBaseUrl(rawBaseUrl: string): string {
  const trimmed = rawBaseUrl.trim();

  if (/^https?:\/\//i.test(trimmed)) {
    return trimmed.replace(/\/+$/, "");
  }

  const basePath = `/${trimmed.replace(/^\/+/, "").replace(/\/+$/, "")}`;
  if (typeof window !== "undefined") {
    return new URL(basePath, window.location.origin)
      .toString()
      .replace(/\/+$/, "");
  }

  // Fallback for non-browser runtimes (tests/tooling).
  return new URL(basePath, "http://localhost").toString().replace(/\/+$/, "");
}

export class ApiClient {
  private baseUrl: string;
  private _token: string | null = null;
  private authRetry = false;

  constructor(baseUrl: string) {
    this.baseUrl = normalizeBaseUrl(baseUrl);
  }

  setToken(token: string | null) {
    this._token = token;
  }

  get token(): string | null {
    return this._token;
  }

  private async request<T>(
    method: string,
    path: string,
    params?: Record<string, string>,
    body?: unknown,
  ): Promise<T> {
    const normalizedPath = path.replace(/^\/+/, "");
    const url = new URL(normalizedPath, `${this.baseUrl}/`);
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        url.searchParams.set(k, v);
      }
    }

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };
    if (this._token) {
      headers["Authorization"] = `Bearer ${this._token}`;
    }

    const res = await fetch(url.toString(), {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    if (res.status === 401 && !this.authRetry) {
      this.authRetry = true;
      try {
        const { silentLogin } = await import("@/stores/auth");
        await silentLogin();
        return this.request<T>(method, path, params, body);
      } finally {
        this.authRetry = false;
      }
    }

    if (!res.ok) {
      const text = await res.text().catch(() => "");
      throw new Error(`API ${res.status}: ${path} — ${text}`);
    }

    const json = (await res.json()) as ApiResponse<T>;
    return json.data;
  }

  async get<T>(path: string, params?: Record<string, string>): Promise<T> {
    return this.request<T>("GET", path, params);
  }

  async post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("POST", path, undefined, body);
  }
}

let client: ApiClient | null = null;

export function getClient(): ApiClient {
  if (!client) {
    const baseUrl = import.meta.env.VITE_API_BASE_URL || "/api/v1";
    client = new ApiClient(baseUrl);
  }
  return client;
}

export function configureClient(baseUrl: string, token?: string): ApiClient {
  client = new ApiClient(baseUrl);
  if (token) client.setToken(token);
  return client;
}
