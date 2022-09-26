import {
    AuthenticityTokenInput,
    AuthenticityTokenProvider,
    createAuthenticityToken,
    useShouldHydrate,
} from "remix-utils";
import {commitSession} from "~/utils/sessions.server";
import {getSessionData} from "./utils/auth.server";
import styles from "./styles/app.css"
import {routes} from "~/routes";
import {ActionFunction, json, LoaderFunction, redirect} from "@remix-run/node";
import {Links, LiveReload, Meta, Outlet, Scripts, ScrollRestoration, useFetcher, useLoaderData} from "@remix-run/react";
import {ClientStyleContext, ServerStyleContext} from './context'
import React, {useContext, useEffect} from "react";
import {withEmotionCache} from "@emotion/react";
import {ChakraProvider} from "@chakra-ui/react";

interface LoaderData {
    csrf?: string;
    isLoggedIn: boolean;
    ENV: {
        FIREBASE_CONFIG?: string
    }

}

export function links() {
    return [{rel: "stylesheet", href: styles}]
}

// Setup CSRF token only if they are heading to the login page.
// Usually we would assign a CSRF token to everyone but
// with Firebase Hosting caching is tied to the cookie.
// If someone isn't logged in we want them to hit the public cache
// https://firebase.google.com/docs/hosting/manage-cache
export const action: ActionFunction = async ({request}) => {
    let {session} = await getSessionData(request);

    // Add CSRF token to session
    createAuthenticityToken(session);

    return redirect("/login", {
        headers: {"Set-Cookie": await commitSession(session)},
    });
};

export const loader: LoaderFunction = async ({request}) => {
    const {csrf, idToken} = await getSessionData(request);
    return json<LoaderData>({
        csrf,
        isLoggedIn: !!idToken,
        ENV: {
            FIREBASE_CONFIG: process.env.FIREBASE_CONFIG,
        }
    });
};


interface DocumentProps {
    children: React.ReactNode;
}

const Document = withEmotionCache(
    ({ children }: DocumentProps, emotionCache) => {
        const serverStyleData = useContext(ServerStyleContext);
        const clientStyleData = useContext(ClientStyleContext);

        // Control if page loads JS https://github.com/sergiodxa/remix-utils#useshouldhydrate
        const shouldHydrate = useShouldHydrate();

        const fetcher = useFetcher();

        const {csrf, isLoggedIn, ENV} = useLoaderData<LoaderData>();


        // Only executed on client
        useEffect(() => {
            // re-link sheet container
            emotionCache.sheet.container = document.head;
            // re-inject tags
            const tags = emotionCache.sheet.tags;
            emotionCache.sheet.flush();
            tags.forEach((tag) => {
                (emotionCache.sheet as any)._insertTag(tag);
            });
            // reset cache to reapply global styles
            clientStyleData?.reset();
        }, []);

        return (
            <html lang="en">
            <head>
                <meta charSet="utf-8"/>
                <meta name="viewport" content="width=device-width,initial-scale=1"/>
                <meta name="robots" content="noindex"/>
                <Meta/>
                <Links/>
                <title>Smart Home</title>
                <Meta />
                <Links />
                {serverStyleData?.map(({ key, ids, css }) => (
                    <style
                        key={key}
                        data-emotion={`${key} ${ids.join(' ')}`}
                        dangerouslySetInnerHTML={{ __html: css }}
                    />
                ))}

            </head>
            <body
                className="mx-1"
            >
            <AuthenticityTokenProvider token={csrf || ""}>
                <nav
                    className="flex align-middle justify-between mx-2"
                >
                    {/* We use fetcher.Form instead of Form because we dont want navigation events */}
                    {isLoggedIn ? (
                        <>
                            <div>
                                <a href={routes.HOME}>Home</a>
                                <a href={routes.PLUGS.ROOT}>Plugs</a>
                                <a href={routes.SCHEDULES.ROOT}>Schedules</a>
                            </div>
                            <fetcher.Form action="/logout" method="post" replace>
                                <AuthenticityTokenInput/>
                                <button type="submit">Logout</button>
                            </fetcher.Form>
                        </>
                    ) : (
                        <>
                            <a href="/">Front page</a>
                            <fetcher.Form action="/" method="post" replace>
                                <button type="submit">Login</button>
                            </fetcher.Form>
                        </>
                    )}
                </nav>
                <script
                    dangerouslySetInnerHTML={{
                        __html: `window.ENV = ${JSON.stringify(
                            ENV
                        )}`,
                    }}
                />
                {children}
            </AuthenticityTokenProvider>
            <ScrollRestoration/>
            {shouldHydrate && <Scripts/>}
            {process.env.NODE_ENV === "development" && <LiveReload/>}
            </body>
            </html>
        );
    }
);


export default function App() {
    return (
        <Document>
            <ChakraProvider>
                <Outlet />
            </ChakraProvider>
        </Document>
    )
}
