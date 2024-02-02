import {cssBundleHref} from "@remix-run/css-bundle";
import type {LinksFunction, LoaderFunctionArgs} from "@remix-run/node";
import {
    Links,
    LiveReload,
    Meta,
    Outlet,
    Scripts,
    ScrollRestoration,
    useLoaderData,
    useRouteError,
} from "@remix-run/react";

import styles from "./tailwind.css";
import Layout from "~/components/layout";
import {Theme, ThemeProvider, useTheme} from "remix-themes"
import {themeSessionResolver} from "~/sessions.server";
import {getErrorComponent} from "~/components/error";


export const links: LinksFunction = () => [
    {rel: "stylesheet", href: styles},
    ...(cssBundleHref ? [{rel: "stylesheet", href: cssBundleHref}] : []),
];

export async function loader({request}: LoaderFunctionArgs) {
    const {getTheme} = await themeSessionResolver(request)
    return {
        theme: getTheme(),
    }
}

export default function AppWithProviders() {
    const data = useLoaderData<typeof loader>()
    return (
        <ThemeProvider specifiedTheme={data.theme} themeAction="/set-theme">
            <App/>
        </ThemeProvider>
    )
}

function App() {
    const [theme] = useTheme()

    return (
        <html lang="en">
        <head>
            <meta charSet="utf-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1"/>
            <Meta/>
            <PwaLinksAndMeta/>
            <Links/>
        </head>
        <body
            className={theme === Theme.DARK ? 'dark' : ''}
        >
        <Layout>
            <Outlet/>
        </Layout>
        <ScrollRestoration/>
        <Scripts/>
        <LiveReload/>
        </body>
        </html>
    );
}

export function ErrorBoundary() {
    const error = useRouteError();

    return (
        <html>
        <head>
            <title>Oops!</title>
            <Meta/>
            <PwaLinksAndMeta/>
            <Links/>
        </head>
        <body>
        {getErrorComponent(error)}
        <Scripts/>
        </body>
        </html>
    );
}

const PwaLinksAndMeta = () => <>
    <link rel="manifest" href="/manifest/webmanifest"/>

    <meta
        name="apple-mobile-web-app-status-bar-style"
        content="black-translucent"
    />
    <meta
        name="viewport"
        content="initial-scale=1, viewport-fit=cover, user-scalable=no"
    />
    <meta name="viewport" content="initial-scale=1, viewport-fit=cover"/>
    {/*
        pwa-asset-generator
        npx pwa-asset-generator app/images/smarthome-logo.svg public/icons -b "linear-gradient(29deg, rgba(2,0,36,1) 0%, rgba(9,43,121,1) 31%, rgba(0,255,93,1) 100%)" -q 100 --favicon
    */}
    <link rel="icon" type="image/png" sizes="196x196" href="/icons/favicon-196.png"/>

    <link rel="apple-touch-icon" href="/icons/apple-icon-180.png"/>

    <meta name="apple-mobile-web-app-capable" content="yes"/>

    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2048-2732.jpg"
          media="(device-width: 1024px) and (device-height: 1366px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2732-2048.jpg"
          media="(device-width: 1024px) and (device-height: 1366px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1668-2388.jpg"
          media="(device-width: 834px) and (device-height: 1194px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2388-1668.jpg"
          media="(device-width: 834px) and (device-height: 1194px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1536-2048.jpg"
          media="(device-width: 768px) and (device-height: 1024px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2048-1536.jpg"
          media="(device-width: 768px) and (device-height: 1024px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1668-2224.jpg"
          media="(device-width: 834px) and (device-height: 1112px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2224-1668.jpg"
          media="(device-width: 834px) and (device-height: 1112px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1620-2160.jpg"
          media="(device-width: 810px) and (device-height: 1080px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2160-1620.jpg"
          media="(device-width: 810px) and (device-height: 1080px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1290-2796.jpg"
          media="(device-width: 430px) and (device-height: 932px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2796-1290.jpg"
          media="(device-width: 430px) and (device-height: 932px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1179-2556.jpg"
          media="(device-width: 393px) and (device-height: 852px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2556-1179.jpg"
          media="(device-width: 393px) and (device-height: 852px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1284-2778.jpg"
          media="(device-width: 428px) and (device-height: 926px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2778-1284.jpg"
          media="(device-width: 428px) and (device-height: 926px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1170-2532.jpg"
          media="(device-width: 390px) and (device-height: 844px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2532-1170.jpg"
          media="(device-width: 390px) and (device-height: 844px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1125-2436.jpg"
          media="(device-width: 375px) and (device-height: 812px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2436-1125.jpg"
          media="(device-width: 375px) and (device-height: 812px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1242-2688.jpg"
          media="(device-width: 414px) and (device-height: 896px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2688-1242.jpg"
          media="(device-width: 414px) and (device-height: 896px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-828-1792.jpg"
          media="(device-width: 414px) and (device-height: 896px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1792-828.jpg"
          media="(device-width: 414px) and (device-height: 896px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1242-2208.jpg"
          media="(device-width: 414px) and (device-height: 736px) and (-webkit-device-pixel-ratio: 3) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-2208-1242.jpg"
          media="(device-width: 414px) and (device-height: 736px) and (-webkit-device-pixel-ratio: 3) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-750-1334.jpg"
          media="(device-width: 375px) and (device-height: 667px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1334-750.jpg"
          media="(device-width: 375px) and (device-height: 667px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-640-1136.jpg"
          media="(device-width: 320px) and (device-height: 568px) and (-webkit-device-pixel-ratio: 2) and (orientation: portrait)"/>
    <link rel="apple-touch-startup-image" href="/icons/apple-splash-1136-640.jpg"
          media="(device-width: 320px) and (device-height: 568px) and (-webkit-device-pixel-ratio: 2) and (orientation: landscape)"/>
</>
