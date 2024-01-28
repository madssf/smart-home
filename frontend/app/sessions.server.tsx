import {createThemeSessionResolver} from "remix-themes"
import {createCookieSessionStorage} from "@remix-run/node";

// const isProduction = process.env.NODE_ENV === "production"

const sessionStorage = createCookieSessionStorage({
    cookie: {
        name: "theme",
        path: "/",
        httpOnly: true,
        sameSite: "lax",
        secrets: ["s3cr3t"],
        // Set domain and secure only if in production
        /* Actually let's try not setting these and see if it works - since we are on two different domains
        ...(isProduction
            ? { domain: "your-production-domain.com", secure: true }
            : {}),
         */
    },
})

export const themeSessionResolver = createThemeSessionResolver(sessionStorage)
