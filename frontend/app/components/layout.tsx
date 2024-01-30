import React from 'react';
import Header from "~/components/header";

export interface LayoutProps {
    children: React.ReactNode
}

const Layout = ({children}: LayoutProps) => {
    return (
        <>
            <Header/>
            <div className="mt-20 pt-safe-top">
                {children}
            </div>
        </>
    );
};

export default Layout;
