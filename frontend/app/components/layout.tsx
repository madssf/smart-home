import React from 'react';
import Nav from "~/components/nav";

export interface LayoutProps {
    children: React.ReactNode
}

const Layout = ({children}: LayoutProps) => {
    return (
        <>
            <Nav/>
            {children}
        </>
    );
};

export default Layout;
