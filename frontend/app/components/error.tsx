import {Button} from "~/components/ui/button";
import {Link} from "@remix-run/react";

export type RouteErrorType = {
    type: 'ROUTE_ERROR',
    status: number,
    statusText: string,
    data: string,
}

export type ApplicationError = {
    type: 'ERROR',
    message: string,
    stack: string,
};

type ErrorType = RouteErrorType | ApplicationError;

type Props = {
    errorType?: ErrorType,
};

export function CustomError(props: Props) {
    return (
        <main className="m-auto flex flex-col items-center">
            <h1 className="my-24 mx-16">Something went wrong!</h1>
            <div
                className="my-24 mx-16"
            >
                {
                    props.errorType ?
                        renderErrorDetails(props.errorType)
                        :
                        "Unknown error"
                }
            </div>
            <Button className="mb-40">
                <Link to="/">
                    Home
                </Link>
            </Button>
        </main>
    );
}

const renderErrorDetails = (errorType: ErrorType) => {
    switch (errorType.type) {
        case 'ROUTE_ERROR':
            return (
                <div>
                    <h2 className="text-24 mb-8">Error details</h2>
                    <p className="text-16 mb-8">Status: {errorType.status}</p>
                    <p className="text-16 mb-8">Status text: {errorType.statusText}</p>
                    <p className="text-16 mb-8">Data: {errorType.data}</p>
                </div>
            );
        case 'ERROR':
            return (
                <div>
                    <h2 className="text-24 mb-8">Error details</h2>
                    <p className="text-16 mb-8">Message: {errorType.message}</p>
                    <p className="text-16 mb-8">Stack: {errorType.stack}</p>
                </div>
            );
    }
}
