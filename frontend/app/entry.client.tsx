import React, {useState} from 'react'
import {CacheProvider} from '@emotion/react'
import {RemixBrowser} from '@remix-run/react'
import {ClientStyleContext} from './context'
import createEmotionCache from './createEmotionCache'
// @ts-ignore
import * as ReactDOMClient from 'react-dom/client'

interface ClientCacheProviderProps {
    children: React.ReactNode;
}

function ClientCacheProvider({ children }: ClientCacheProviderProps) {
    const [cache, setCache] = useState(createEmotionCache())

    function reset() {
        setCache(createEmotionCache())
    }

    return (
        <ClientStyleContext.Provider value={{ reset }}>
            <CacheProvider value={cache}>{children}</CacheProvider>
        </ClientStyleContext.Provider>
    )
}

ReactDOMClient.hydrateRoot(
    document,
    <ClientCacheProvider>
        <RemixBrowser />
    </ClientCacheProvider>,
)