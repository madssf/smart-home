import {useEffect} from "react";
import {ClientOnly, unauthorized, useHydrated} from "remix-utils";
import {commitSession} from "~/utils/sessions.server";
import {admin} from "~/utils/firebase.server";
import {getSessionData} from "~/utils/auth.server";
import firebase from "firebase/compat/app";
import "firebase/compat/auth";
import StyledFirebaseAuth from "~/components/styledFirebaseAuth";
import type {ActionFunction} from "@remix-run/node";
import {redirect} from "@remix-run/node";
import {useSubmit} from "@remix-run/react";

// Extend window

declare global {
    interface Window {
        ENV: any;
    }
}

// We need Javascript client side to run the Firebase Login component
export const handle = {hydrate: true};

export const action: ActionFunction = async ({request}) => {
    const {session} = await getSessionData(request);
    const form = await request.formData();
    const idToken = form.get("idToken") as string;
    // Set session expiration to 5 days.
    const expiresIn = 60 * 60 * 24 * 5 * 1000;
    try {
        const token = await admin.auth().verifyIdToken(idToken);

        if (new Date().getTime() / 1000 - token.auth_time < 5 * 60) {
            // Create session cookie and set it.
            const cookie = await admin
                .auth()
                .createSessionCookie(idToken, {expiresIn});
            session.set("idToken", cookie);
            return redirect("/home", {
                headers: {"Set-Cookie": await commitSession(session)},
            });
        }
        // If the JWT is too old we reject it
        await admin.auth().revokeRefreshTokens(token.sub);
        return new Response("Recent sign in required!", {
            status: 401,
        });
    } catch (error) {
        throw unauthorized("Token invalid");
    }
};
export default function Login() {
    const submit = useSubmit();
    // Check if we are in the browser or server
    const hydrated = useHydrated();

    useEffect(() => {
        if (!firebase.apps.length)
            firebase.initializeApp(JSON.parse(atob(window.ENV.FIREBASE_CONFIG)));
        // Our auth is persisted in our cookie
        firebase.auth().setPersistence(firebase.auth.Auth.Persistence.NONE);
    }, []);

    return (
        <ClientOnly>
            {() => (<StyledFirebaseAuth
                uiConfig={{
                    // Popup signin flow rather than redirect flow.
                    signInFlow: "redirect",
                    callbacks: {
                        // On sign in we POST our server with the JWT token
                        signInSuccessWithAuthResult: (
                            authResult: firebase.auth.UserCredential,
                        ) => {
                            authResult.user?.getIdToken().then((idToken) => {
                                const formData = new FormData();
                                formData.append("idToken", idToken);
                                submit(formData, {
                                    method: "post",
                                    action: "/login",
                                    // Don't create entry on browser history stack
                                    replace: true,
                                });
                            });
                            return false;
                        },
                    },
                    signInOptions: [
                        firebase.auth.GoogleAuthProvider.PROVIDER_ID,
                        {
                            provider: firebase.auth.EmailAuthProvider.PROVIDER_ID,
                            requireDisplayName: false,
                        },
                    ],
                }}
                firebaseAuth={firebase.auth()}
            ></StyledFirebaseAuth>)}

        </ClientOnly>
    );
}
