import {
    json,
    Links,
    LiveReload,
    Meta,
    Outlet,
    Scripts,
    ScrollRestoration,
    LoaderFunction,
    useLoaderData,
    ActionFunction,
    redirect,
    Form,
    useFetcher,
} from "remix";
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

export default function App() {
    // Control if page loads JS https://github.com/sergiodxa/remix-utils#useshouldhydrate
    const shouldHydrate = useShouldHydrate();

    const fetcher = useFetcher();

    const {csrf, isLoggedIn, ENV} = useLoaderData<LoaderData>();

    return (
        <html lang="en">
        <head>
            <meta charSet="utf-8"/>
            <meta name="viewport" content="width=device-width,initial-scale=1"/>
            <meta name="robots" content="noindex"/>
            <Meta/>
            <Links/>
            <title>Smart Home</title>
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
            <Outlet/>
            <script
                dangerouslySetInnerHTML={{
                    __html: `window.ENV = ${JSON.stringify(
                        ENV
                    )}`,
                }}
            />
        </AuthenticityTokenProvider>
        <ScrollRestoration/>
        {shouldHydrate && <Scripts/>}
        {process.env.NODE_ENV === "development" && <LiveReload/>}
        </body>
        </html>
    );
}
