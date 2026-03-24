interface Env {
  BACKEND_URL: string;
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

    // Static assets are served automatically by the assets config.
    // If we get here, it means no static file matched and no API route matched.
    return new Response("Not Found", { status: 404 });
  },
};
