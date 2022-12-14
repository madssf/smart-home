import {useShouldHydrate} from "remix-utils";
import styles from "./styles/app.css";
import {Links, LiveReload, Meta, Outlet, Scripts, ScrollRestoration, useNavigate} from "@remix-run/react";
import {ClientStyleContext, ServerStyleContext} from './context';
import React, {useContext, useEffect} from "react";
import {withEmotionCache} from "@emotion/react";
import {Box, Button, ButtonGroup, extendTheme, Heading, withDefaultColorScheme} from "@chakra-ui/react";
import Layout from "~/components/layout";
import {routes} from "~/routes";

export const theme = extendTheme(
    withDefaultColorScheme({ colorScheme: 'teal' }),
);

export function links() {
    return [{rel: "stylesheet", href: styles}];
}

interface DocumentProps {
    children: React.ReactNode;
}

const Document = withEmotionCache(
    ({ children }: DocumentProps, emotionCache) => {
        const serverStyleData = useContext(ServerStyleContext);
        const clientStyleData = useContext(ClientStyleContext);

        // Control if page loads JS https://github.com/sergiodxa/remix-utils#useshouldhydrate
        const shouldHydrate = useShouldHydrate();

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
            // eslint-disable-next-line react-hooks/exhaustive-deps
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
            <body>
            {children}
            <ScrollRestoration/>
            {shouldHydrate && <Scripts/>}
            {process.env.NODE_ENV === "development" && <LiveReload/>}
            </body>
            </html>
        );
    },
);


export default function App() {

    return (
        <Document>
            <Layout>
                <Box className="mx-3 pb-8">
                    <Outlet />
                </Box>
            </Layout>
        </Document>
    );
}


export function ErrorBoundary({error}: { error: Error }) {
    console.error(error);

    const navigate = useNavigate();

    return (
        <Document>
            <Layout>
                <Box className="flex flex-col items-center text-center">

                    <Heading size="md" pb={4}>
                        Oops!
                    </Heading>

                    <p className="text-center pb-4">{error.message}</p>
                    <ButtonGroup>
                        <Button p={1} onClick={() => window.location.reload()}>
                            Reload
                        </Button>
                        <Button p={1} variant="outline" onClick={() => navigate(routes.HOME)}>
                            Home
                        </Button>
                    </ButtonGroup>
                </Box>
            </Layout>
        </Document>

    );
}
