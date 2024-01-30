import {json} from '@remix-run/node';

export const loader = async () => {

    return json(
        {
            short_name: 'Smart Home',
            name: 'Smart Home',
            start_url: '/',
            display: 'standalone',
            icons: [
                {
                    "src": "/icons/manifest-icon-192.maskable.png",
                    "sizes": "192x192",
                    "type": "image/png",
                    "purpose": "any"
                },
                {
                    "src": "/icons/manifest-icon-192.maskable.png",
                    "sizes": "192x192",
                    "type": "image/png",
                    "purpose": "maskable"
                },
                {
                    "src": "/icons/manifest-icon-512.maskable.png",
                    "sizes": "512x512",
                    "type": "image/png",
                    "purpose": "any"
                },
                {
                    "src": "/icons/manifest-icon-512.maskable.png",
                    "sizes": "512x512",
                    "type": "image/png",
                    "purpose": "maskable"
                }
            ],
        },
        {
            headers: {
                'Cache-Control': 'public, max-age=600',
                'Content-Type': 'application/manifest+json',
            },
        }
    );
};
