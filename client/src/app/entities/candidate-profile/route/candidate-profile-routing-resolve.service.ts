import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { ICandidateProfile } from '../candidate-profile.model';
import { CandidateProfileService } from '../service/candidate-profile.service';

const candidateProfileResolve = (route: ActivatedRouteSnapshot): Observable<null | ICandidateProfile> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(CandidateProfileService);
    return service.find(id).pipe(
      catchError((error: HttpErrorResponse) => {
        if (error.status === 404) {
          router.navigate(['404']);
        } else {
          router.navigate(['error']);
        }
        return EMPTY;
      }),
    );
  }

  return of(null);
};

export default candidateProfileResolve;
