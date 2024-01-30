import {json, LoaderFunctionArgs} from '@remix-run/node';
import {themeSessionResolver} from "~/sessions.server";

export const loader = async ({request} : LoaderFunctionArgs) => {
  const { getTheme } = await themeSessionResolver(request)

  const theme = getTheme()

  const background_color = theme === 'dark' ? 'hsl(222.2, 84%, 4.9%)' : 'hsl(0, 0%, 100%)'
  const theme_color = background_color

  return json(
    {
      short_name: 'Smart Home',
      name: 'Smart Home',
      start_url: '/',
      display: 'standalone',
      background_color,
      theme_color,
      shortcuts: [
        {
          name: 'Homepage',
          url: '/',
          icons: [
            {
              src: '/icons/android-icon-96x96.png',
              sizes: '96x96',
              type: 'image/png',
              purpose: 'any monochrome',
            },
          ],
        },
      ],
      icons: [
        {
          src: '/icons/android-icon-36x36.png',
          sizes: '36x36',
          type: 'image/png',
          density: '0.75',
        },
        {
          src: '/icons/android-icon-48x48.png',
          sizes: '48x48',
          type: 'image/png',
          density: '1.0',
        },
        {
          src: '/icons/android-icon-72x72.png',
          sizes: '72x72',
          type: 'image/png',
          density: '1.5',
        },
        {
          src: '/icons/android-icon-96x96.png',
          sizes: '96x96',
          type: 'image/png',
          density: '2.0',
        },
        {
          src: '/icons/android-icon-144x144.png',
          sizes: '144x144',
          type: 'image/png',
          density: '3.0',
        },
        {
          src: '/icons/android-chrome-192x192.png',
          sizes: '192x192',
          type: 'image/png',
        },
        {
          src: '/icons/android-chrome-256x256.png',
          sizes: '256x256',
          type: 'image/png',
        },
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
