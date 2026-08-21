import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import RadarStateResolve from './route/radar-state-routing-resolve.service';

const radarStateRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/radar-state').then(m => m.RadarState),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/radar-state-detail').then(m => m.RadarStateDetail),
    resolve: {
      radarState: RadarStateResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/radar-state-update').then(m => m.RadarStateUpdate),
    resolve: {
      radarState: RadarStateResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/radar-state-update').then(m => m.RadarStateUpdate),
    resolve: {
      radarState: RadarStateResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default radarStateRoute;
