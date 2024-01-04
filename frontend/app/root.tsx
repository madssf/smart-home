import {cssBundleHref} from "@remix-run/css-bundle";
import type {LinksFunction} from "@remix-run/node";
import {Links, LiveReload, Meta, Outlet, Scripts, ScrollRestoration, useNavigate,} from "@remix-run/react";

import styles from "./tailwind.css";
import Layout from "~/components/layout";

export const links: LinksFunction = () => [
    {rel: "stylesheet", href: styles},
    ...(cssBundleHref ? [{rel: "stylesheet", href: cssBundleHref}] : []),
];

export default function App() {
    return (
        <html lang="en">
        <head>
            <meta charSet="utf-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1"/>
            <Meta/>
            <Links/>
        </head>
        <body>
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

export function ErrorBoundary({error}: { error: Error }) {

    const navigate = useNavigate();

    return <html lang="en">
    <head>
        <meta charSet="utf-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta/>
        <Links/>
    </head>
    <body>
    <Layout>
        <h1>Oh no, something went wrong!</h1>
        <pre>{error.message}</pre>
        <button onClick={() => navigate('/')}>Go Home</button>
    </Layout>
    <Scripts/>
    <LiveReload/>
    </body>
    </html>;

}
