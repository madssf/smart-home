import React from 'react';
import {
    Button,
    IconButton,
    Link,
    Menu,
    MenuButton,
    MenuDivider,
    MenuItem,
    MenuList,
    useColorMode,
} from "@chakra-ui/react";
import {routes} from "~/routes";
import {useFetcher} from "@remix-run/react";
import {HamburgerIcon, MoonIcon, SunIcon} from "@chakra-ui/icons";
import {homeLink, pageLinks} from './pageLinks';
import {useLocation} from "react-router-dom";

export interface NavProps {
    isLoggedIn?: boolean
}

const Nav = ({isLoggedIn}: NavProps) => {
    const loggedIn = isLoggedIn ?? true;
    const { colorMode, toggleColorMode } = useColorMode();
    const fetcher = useFetcher();
    const location = useLocation();

    return (
        <nav
            className="flex align-middle justify-between m-3"
        >

            <div>
                {loggedIn ?
                    <>
                        <Menu>
                            <MenuButton
                                as={IconButton}
                                aria-label='Navigation'
                                icon={<HamburgerIcon />}
                                variant='outline'
                                size="sm"
                            />
                            <MenuList>
                                {
                                    location.pathname?.endsWith("/home") ?
                                        null
                                        :
                                        <>
                                            <MenuItem>
                                                {homeLink}
                                            </MenuItem>
                                            <MenuDivider />
                                        </>

                                }
                                {pageLinks.map((link) => {
                                    return (
                                        <MenuItem key={link.key}>
                                            {link}
                                        </MenuItem>
                                    );
                                })}
                            </MenuList>
                        </Menu>
                    </>
                    :
                    <Link className="mr-2" href={routes.ROOT}>Front page</Link>
                }
            </div>
            <div className="flex">
                <Button className="mr-4" size="sm" onClick={toggleColorMode} variant='outline'>
                    {colorMode === 'light' ? <MoonIcon /> : <SunIcon />}
                </Button>
                {
                    loggedIn ?
                        <fetcher.Form action="/logout" method="post" replace>
                            <Button variant="outline" size="sm" type="submit">Logout</Button>
                        </fetcher.Form>
                        :
                        <fetcher.Form action="/" method="post" replace>
                            <Button variant="outline" size="sm" type="submit">Login</Button>
                        </fetcher.Form>
                }

            </div>
        </nav>


    );
};

export default Nav;
