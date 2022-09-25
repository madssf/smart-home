import React from 'react';
import {Link, Outlet, useLocation} from "remix";
import {routes} from "~/routes";

const Plugs = () => {

    const location = useLocation()
    return (
        <div>
            Plugs!
            {
                location.pathname !== routes.PLUGS.NEW ?
                    <Link to={routes.PLUGS.NEW}>Add plug</Link>
                    :
                    <Link to={routes.PLUGS.ROOT}>Cancel</Link>
            }
            <Outlet />
        </div>
    );
};

export default Plugs;