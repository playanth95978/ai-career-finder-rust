import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import UserPreferenceResolve from './route/user-preference-routing-resolve.service';

const userPreferenceRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/user-preference').then(m => m.UserPreference),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/user-preference-detail').then(m => m.UserPreferenceDetail),
    resolve: {
      userPreference: UserPreferenceResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/user-preference-update').then(m => m.UserPreferenceUpdate),
    resolve: {
      userPreference: UserPreferenceResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/user-preference-update').then(m => m.UserPreferenceUpdate),
    resolve: {
      userPreference: UserPreferenceResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default userPreferenceRoute;
