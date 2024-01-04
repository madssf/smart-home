import React from 'react';
import {homeLink, pageLinks} from './pageLinks';
import {useLocation} from '@remix-run/react';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import {AlignJustify, Moon} from "lucide-react";
import {Button} from "~/components/ui/button";


const Nav = () => {
    const location = useLocation();

    return (
        <nav
            className="flex align-middle justify-between m-3"
        >

            <div>
                <DropdownMenu>
                    <DropdownMenuTrigger>
                        <AlignJustify />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        {
                            location.pathname?.endsWith("/home") ?
                                null
                                :
                                <>
                                    <DropdownMenuLabel>
                                        {homeLink}
                                    </DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                </>

                        }
                        {pageLinks.map((link) => {
                            return (
                                <DropdownMenuLabel key={link.key}>
                                    {link}
                                </DropdownMenuLabel>
                            );
                        })}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
            <div className="flex">
                <Button size="sm" onClick={() => {
                    // TODO: Re-enable dark mode
                    console.log("TODO: Re-enable dark mode");
                }} variant='outline'>
                    <Moon />
                </Button>
            </div>
        </nav>


    );
};

export default Nav;
