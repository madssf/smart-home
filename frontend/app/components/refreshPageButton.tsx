import {Button} from "~/components/ui/button";
import {RefreshCw} from "lucide-react";

const RefreshPageButton = () => {
    return (
        <Button variant="outline" size="icon" onClick={() => window.location.reload()}>
            <RefreshCw className="h-[1.2rem] w-[1.2rem]" />
        </Button>
    );
};

export default RefreshPageButton;
