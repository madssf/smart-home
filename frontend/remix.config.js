import {createRoutesFromFolders,} from "@remix-run/v1-route-convention";

/**
 * @type {import('@remix-run/dev').AppConfig}
 */
export default {
    ignoredRouteFiles: [".*"],
    // appDirectory: "app",
    // assetsBuildDirectory: "public/build",
    // serverBuildPath: "build/index.js",
    // publicPath: "/build/",
    // devServerPort: 8002,
    serverDependenciesToBundle: [
        /^remix-utils.*/,
    ],
    tailwind: true,
    postcss: true,
    routes(defineRoutes) {
        // uses the v1 convention, works in v1.15+ and v2
        return createRoutesFromFolders(defineRoutes);
    },
};
