import {cssBundleHref} from "@remix-run/css-bundle";
import type {LinksFunction, LoaderFunctionArgs} from "@remix-run/node";
import {
    isRouteErrorResponse,
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
import {ApplicationError, CustomError, RouteErrorType} from "~/components/error";


export const links: LinksFunction = () => [
    {rel: "stylesheet", href: styles},
    ...(cssBundleHref ? [{rel: "stylesheet", href: cssBundleHref}] : []),
];

export async function loader({ request }: LoaderFunctionArgs) {
    const { getTheme } = await themeSessionResolver(request)
    return {
        theme: getTheme(),
    }
}

export default function AppWithProviders() {
    const data = useLoaderData<typeof loader>()
    return (
        <ThemeProvider specifiedTheme={data.theme} themeAction="/set-theme">
            <App />
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
            <Meta />
            <Links />
        </head>
        <body>
        {getErrorComponent(error)}
        <Scripts />
        </body>
        </html>
    );
}

const getErrorComponent = (error: unknown) => {
    if (isRouteErrorResponse(error)) {
        const errorType: RouteErrorType = {
            type: 'ROUTE_ERROR',
            status: error.status,
            statusText: error.statusText,
            data: error.data,
        }
        return (
            <CustomError errorType={errorType}  />
        );
    } else if (error instanceof Error) {
        const errorType: ApplicationError = {
            type: 'ERROR',
            message: error.message,
            stack: error.stack ?? '',
        }
        return (
            <CustomError errorType={errorType} />
        );
    } else {
        return <CustomError />;
    }
}
