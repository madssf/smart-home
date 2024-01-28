import {useLocation} from '@remix-run/react';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import {AlignJustify} from "lucide-react";
import {homeLink, pageLinks} from "~/components/pageLinks";
import {ThemePicker} from "~/components/themePicker";


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
                <ThemePicker />
            </div>
        </nav>


    );
};

export default Nav;
