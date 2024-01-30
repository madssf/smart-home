import {Button} from "~/components/ui/button";
import {isRouteErrorResponse, Link} from "@remix-run/react";

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
            <h1 className="my-12 mx-16">Something went wrong!</h1>
            <div
                className="my-12 mx-16"
            >
                {
                    props.errorType ?
                        renderErrorDetails(props.errorType)
                        :
                        "Unknown error"
                }
            </div>
            <Button className="mb-24">
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
                    <h2 className="font-bold mb-4">Error details</h2>
                    <p className="text-sm mb-2">Status: {errorType.status}</p>
                    <p className="text-sm mb-2">Status text: {errorType.statusText}</p>
                    <p className="text-sm mb-2">Data: {errorType.data}</p>
                </div>
            );
        case 'ERROR':
            return (
                <div>
                    <h2 className="font-bold mb-4">Error details</h2>
                    <p className="text-sm mb-2">Message: {errorType.message}</p>
                    <p className="text-sm mb-2">Stack: {errorType.stack}</p>
                </div>
            );
    }
}

export const getErrorComponent = (error: unknown) => {
    if (isRouteErrorResponse(error)) {
        const errorType: RouteErrorType = {
            type: 'ROUTE_ERROR',
            status: error.status,
            statusText: error.statusText,
            data: error.data,
        }
        return (
            <CustomError errorType={errorType}  />
        );
    } else if (error instanceof Error) {
        const errorType: ApplicationError = {
            type: 'ERROR',
            message: error.message,
            stack: error.stack ?? '',
        }
        return (
            <CustomError errorType={errorType} />
        );
    } else {
        return <CustomError />;
    }
}
