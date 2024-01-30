import {Link} from '@remix-run/react';
import {AlignJustify} from "lucide-react";
import {homeLink, pageLinks} from "~/components/pageLinks";
import {ThemePicker} from "~/components/themePicker";

import {cn} from "~/lib/utils"
import {
    NavigationMenu,
    NavigationMenuContent,
    NavigationMenuItem,
    NavigationMenuLink,
    NavigationMenuList,
    NavigationMenuTrigger,
} from "~/components/ui/navigation-menu"
import * as React from "react"

const Header = () => {
    return (
        <header
            className="fixed top-0 pt-safe-top flex flex-row justify-between items-center w-full h-16 px-4 bg-accent-foreground z-50"
        >
            <div>
                <NavMenu/>
            </div>
            <p
                className="text-2xl font-bold text-accent"
            >
                Smart Home
            </p>
            <div className="flex">
                <ThemePicker/>
            </div>
        </header>


    );
};


function NavMenu() {
    return (
        <NavigationMenu>
            <NavigationMenuList>
                <NavigationMenuItem>

                    <NavigationMenuTrigger>
                        <AlignJustify/>
                    </NavigationMenuTrigger>
                    <NavigationMenuContent>
                        <div className="w-max px-4 py-2">
                            <ListItem
                                title={homeLink.name}
                                href={homeLink.key}
                            />
                        </div>
                        <hr className="border-accent mx-2"/>
                        <ul className="grid w-max gap-3 p-4 md:grid-cols-2 ">
                            {pageLinks.map((component) => (
                                <li key={component.key}>
                                    <ListItem
                                        title={component.name}
                                        href={component.key}
                                    />
                                </li>
                            ))}
                        </ul>
                    </NavigationMenuContent>
                </NavigationMenuItem>
            </NavigationMenuList>
        </NavigationMenu>
    )
}

const ListItem = React.forwardRef<
    React.ElementRef<"a">,
    React.ComponentPropsWithoutRef<"a">
>(({className, title, href, children, ...props}, ref) => {
    return (
        <NavigationMenuLink asChild>
            <Link
                ref={ref}
                className={cn(
                    "block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground",
                    className
                )}
                to={href ?? "/"}
                {...props}
            >
                <div className="text-sm font-medium leading-none">{title}</div>
            </Link>
        </NavigationMenuLink>
    )
})
ListItem.displayName = "ListItem"


export default Header;
