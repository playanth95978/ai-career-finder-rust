import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import AutoApplyConfigResolve from './route/auto-apply-config-routing-resolve.service';

const autoApplyConfigRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/auto-apply-config').then(m => m.AutoApplyConfig),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/auto-apply-config-detail').then(m => m.AutoApplyConfigDetail),
    resolve: {
      autoApplyConfig: AutoApplyConfigResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/auto-apply-config-update').then(m => m.AutoApplyConfigUpdate),
    resolve: {
      autoApplyConfig: AutoApplyConfigResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/auto-apply-config-update').then(m => m.AutoApplyConfigUpdate),
    resolve: {
      autoApplyConfig: AutoApplyConfigResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default autoApplyConfigRoute;
