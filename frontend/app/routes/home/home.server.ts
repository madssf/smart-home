import {getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {Price} from "~/routes/home/types";

export async function getCurrentPrice(): Promise<Price> {
    return await getRequest<Price>(apiRoutes.prices.current);
}
