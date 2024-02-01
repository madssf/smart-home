import React from 'react';
import Header from "~/components/header";

export interface LayoutProps {
    children: React.ReactNode
}

const Layout = ({children}: LayoutProps) => {
    return (
        <>
            <Header/>
            <div className="mt-16 pt-safe-top mx-1 pb-16 min-h-screen">
                {children}
            </div>
        </>
    );
};

export default Layout;
