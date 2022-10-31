import React from 'react';
import {Button, IconButton, Menu, MenuButton, MenuDivider, MenuItem, MenuList, useColorMode} from "@chakra-ui/react";
import {HamburgerIcon, MoonIcon, SunIcon} from "@chakra-ui/icons";
import {homeLink, pageLinks} from './pageLinks';
import {useLocation} from '@remix-run/react';


const Nav = () => {
    const { colorMode, toggleColorMode } = useColorMode();
    const location = useLocation();

    return (
        <nav
            className="flex align-middle justify-between m-3"
        >

            <div>
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
            </div>
            <div className="flex">
                <Button size="sm" onClick={toggleColorMode} variant='outline'>
                    {colorMode === 'light' ? <MoonIcon /> : <SunIcon />}
                </Button>
            </div>
        </nav>


    );
};

export default Nav;
