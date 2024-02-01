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
import RefreshPageButton from "~/components/refreshPageButton";

const Header = () => {
    return <>
        <header
            className="fixed top-0 w-full pb-2 px-2 header-background z-50 pt-safe-top grid grid-cols-3 items-center"
        >
            <div
                className="flex justify-start pt-2" // Added padding to align with the center text margin.
            >
                <NavMenu/>
            </div>
            <p
                className="text-xl sm:text-2xl pt-2 font-bold text-white justify-self-center self-center whitespace-nowrap"
            >
                Smart Home
            </p>
            <div className="flex justify-end pt-2 gap-1">
                <RefreshPageButton/>
                <ThemePicker/>
            </div>
        </header>
    </>;
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
                <div className="font-medium leading-none">{title}</div>
            </Link>
        </NavigationMenuLink>
    )
})
ListItem.displayName = "ListItem"


export default Header;
