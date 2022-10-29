import React from 'react';
import Nav from "~/components/nav";
import {ChakraProvider, ColorModeScript} from "@chakra-ui/react";
import {theme} from "~/root";

export interface LayoutProps {
    children: React.ReactNode
}

const Layout = ({children}: LayoutProps) => {
    return (
    <>
        <ColorModeScript initialColorMode={'dark'} />
        <ChakraProvider theme={theme}>
            <Nav />
            {children}
       </ChakraProvider>
    </>
    );
};

export default Layout;
