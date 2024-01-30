import {Moon, Sun} from "lucide-react"
import {Theme, useTheme} from "remix-themes"

import {Button} from "./ui/button"

export function ThemePicker() {
    const [theme, setTheme] = useTheme()

    return (
        <Button variant="outline" size="icon" onClick={() => setTheme(theme === Theme.LIGHT ? Theme.DARK : Theme.LIGHT)}>
        {
            theme === Theme.LIGHT ?
                <Moon className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
                :
                <Sun className="h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />

        }
        </Button>
    )
}
