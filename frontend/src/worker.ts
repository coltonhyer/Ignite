interface Env {
  BACKEND_URL: string;
  ASSETS: { fetch: (req: Request) => Promise<Response> };
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname.startsWith("/api/")) {
      let backendProxy = new URL(url.pathname + url.search, env.BACKEND_URL).toString();
      return fetch(
        backendProxy,
        request,
      );
    }

    // Hand off all other requests to the native Assets binding.
    // Thanks to `not_found_handling: "single-page-application"` in wrangler.jsonc,
    // this will properly serve the SPA index.html for unknown paths!
    return env.ASSETS.fetch(request);
  },
};
