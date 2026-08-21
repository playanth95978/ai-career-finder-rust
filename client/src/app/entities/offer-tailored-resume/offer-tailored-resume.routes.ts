import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import OfferTailoredResumeResolve from './route/offer-tailored-resume-routing-resolve.service';

const offerTailoredResumeRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/offer-tailored-resume').then(m => m.OfferTailoredResume),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/offer-tailored-resume-detail').then(m => m.OfferTailoredResumeDetail),
    resolve: {
      offerTailoredResume: OfferTailoredResumeResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/offer-tailored-resume-update').then(m => m.OfferTailoredResumeUpdate),
    resolve: {
      offerTailoredResume: OfferTailoredResumeResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/offer-tailored-resume-update').then(m => m.OfferTailoredResumeUpdate),
    resolve: {
      offerTailoredResume: OfferTailoredResumeResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default offerTailoredResumeRoute;
